use dasp::frame::Frame;
use dasp::Sample;
use dasp_ring_buffer::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DelayLine<S> {
    in_point: usize,
    out_point: usize,
    data: S,
}

impl<S> DelayLine<S>
where
    S: Slice,
    S::Element: Copy,
{
    /// The capacity of the delay line (maximum possible delay)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.slice().len() - 1
    }

    pub fn tick(&mut self, item: S::Element) -> S::Element
    where
        S: SliceMut,
    {
        //write before read. In this way we can easily have a delay of 0,
        //but our capacity becomes one less than the length of the data array
        self.data.slice_mut()[self.in_point] = item;
        self.in_point = (self.in_point + 1) % self.data.slice().len();

        let out = self.data.slice_mut()[self.out_point];
        self.out_point = (self.out_point + 1) % self.data.slice().len();

        return out;
    }

    /// Borrows the item at the given index relative to the input (0 is the last input value)
    pub fn tap(&self, index: usize) -> S::Element {
        assert!(index + 1 < self.data.slice().len());

        let wrapped_index: usize;

        if index + 1 > self.in_point {
            wrapped_index = self.data.slice().len() - (index + 1 - self.in_point);
        } else {
            wrapped_index = self.in_point - (index + 1);
        }

        self.data.slice()[wrapped_index]
    }

    pub fn get_delay(&self) -> usize {
        if self.in_point >= self.out_point {
            return self.in_point - self.out_point;
        } else {
            return self.data.slice().len() - (self.out_point - self.in_point);
        }
    }

    /// Borrows the item at the given index relative to the output (0 is previously output value)
    pub fn tap_output(&self, index: usize) -> S::Element {
        assert!(index + 1 < self.data.slice().len());

        let wrapped_index: usize;

        if index + 1 > self.out_point {
            wrapped_index = self.data.slice().len() - (index + 1 - self.out_point);
        } else {
            wrapped_index = self.out_point - (index + 1);
        }

        self.data.slice()[wrapped_index]
    }

    pub fn set_delay(&mut self, delay: usize) {
        assert!(delay <= self.capacity());

        if delay > self.in_point {
            self.out_point = self.data.slice().len() - (delay - self.in_point);
        } else {
            self.out_point = self.in_point - delay;
        }
    }

    pub fn new(data: S, delay: usize) -> Self {
        assert!(data.slice().len() > 1);
        assert!(delay <= data.slice().len() - 1);

        DelayLine {
            in_point: 0,
            out_point: (data.slice().len() - delay) % data.slice().len(),
            data: data,
        }
    }
}

pub struct DelayLineFracLin<T>
where
    T: Slice,
{
    delay_line: DelayLine<T>,
    fractional_delay_part: f64,
}

impl<T> DelayLineFracLin<T>
where
    T: Slice,
    T::Element: Frame,
{
    pub fn new(data: T, delay: f64) -> Self {
        assert!(data.slice().len() > 0);

        let integer_part = delay.trunc() as usize;
        let fractional_part = delay.fract();

        DelayLineFracLin {
            delay_line: DelayLine::new(data, integer_part),
            fractional_delay_part: fractional_part,
        }
    }

    pub fn tick(&mut self, item: T::Element) -> T::Element
    where
        T: SliceMut,
    {
        let out_integer = self.delay_line.tick(item);
        let out_integer_part =
            out_integer.scale_amp((1.0 - self.fractional_delay_part).to_sample());

        let out_frac = if approx::relative_eq!(self.fractional_delay_part, 0.0) {
            //if the delay is exactly equal to the maximum delay, we cannot tap at one past the output
            //but in that case the fractional part is zero, so the fractional output does not matter anyway
            //and we can just set it to anything
            T::Element::EQUILIBRIUM
        } else {
            self.delay_line.tap_output(1)
        };
        let out_frac_part = out_frac.scale_amp(self.fractional_delay_part.to_sample());

        out_integer_part.add_amp(out_frac_part.to_signed_frame())
    }

    pub fn tap_output(&self) -> T::Element {
        self.delay_line.tap_output(0)
    }

    pub fn set_delay(&mut self, delay: f64) {
        let integer_part = delay.trunc() as usize;
        let fractional_part = delay.fract();

        self.delay_line.set_delay(integer_part);
        self.fractional_delay_part = fractional_part;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rand::distributions::{Distribution, Uniform};

    #[test]
    pub fn zero_integer_delay() {
        let mut d = DelayLine::new(vec![0; 100], 0);

        for n in 0..1000 {
            let v = d.tick(n);

            assert_eq!(v, n);
        }
    }

    #[test]
    pub fn fixed_integer_delay() {
        let delay = 5;

        let mut d = DelayLine::new(vec![0; 100], delay);

        for n in 0..12345 {
            let v = d.tick(n);

            if n >= delay {
                //after the transient phase we expect values we previously put in
                assert_eq!(v, n - delay);
            } else {
                //before it we expect 0, which is what the delay line is initialized with
                assert_eq!(v, 0);
            }
        }
    }

    #[test]
    pub fn variable_integer_delay() {
        let mut delay = 5;

        let mut d = DelayLine::new(vec![0; 100], delay);

        for n in 0..123456 {
            if n % 12 == 0 {
                //Every 12th iteration we change the delay
                delay = (delay + 3) % 13;
                d.set_delay(delay);
            }

            let v = d.tick(n);

            if n >= delay {
                //after the transient phase we expect values we previously put in
                assert_eq!(v, n - delay);
            } else {
                //before it we expect 0, which is what the delay line is initialized with
                assert_eq!(v, 0);
            }
        }
    }

    #[test]
    pub fn fixed_integer_taps() {
        let delay = 5;
        let mut d = DelayLine::new(vec![0; 100], delay);

        for n in 0..12345 {
            d.tick(n);

            let taps = std::cmp::min(n + 1, delay);
            for i in 0..taps {
                let t = d.tap(i);
                assert_eq!(t, n - i);
            }
        }
    }

    #[test]
    pub fn integer_max_delay1() {
        DelayLine::new(vec![0; 100], 99);
    }

    #[test]
    pub fn integer_max_delay2() {
        let mut d = DelayLine::new(vec![0; 100], 95);
        d.set_delay(99);
        d.tick(0);
    }

    #[test]
    #[should_panic]
    pub fn integer_delay_too_big1() {
        let mut d = DelayLine::new(vec![0; 100], 100);
        d.tick(0);
    }

    #[test]
    #[should_panic]
    pub fn integer_delay_too_big2() {
        let mut d = DelayLine::new(vec![0; 100], 5);
        d.set_delay(100);
    }

    #[test]
    pub fn zero_frac_delay() {
        let mut d = DelayLineFracLin::new(vec![0; 100], 0.0);

        for n in 0..1000 {
            let v = d.tick(n);

            assert_eq!(v, n);
        }
    }

    #[test]
    pub fn fixed_frac_delay() {
        let delay = 9.4;

        let mut d = DelayLineFracLin::new(vec![0.0; 100], delay);

        for n in 0..12345 {
            let float_n = n as f64;

            let v = d.tick(float_n);

            if float_n >= delay.ceil() {
                //after the transient phase we expect values interpolated between values we previously put in
                let out_a = float_n - delay.floor();
                let out_b = float_n - delay.ceil();

                let expected_out = out_a * (1.0 - delay.fract()) + out_b * delay.fract();

                assert_relative_eq!(v, expected_out);
            } else if float_n <= delay.floor() {
                //At the beginning we should see zeroes
                assert_relative_eq!(v, 0.0);
            }
            //In between these two cases we are interpolating between initial values and values we entered.
            //not checking these for now.
        }
    }

    #[test]
    pub fn variable_frac_delay() {
        let mut delay = 90.0;

        let mut d = DelayLineFracLin::new(vec![0.0; 100], delay);
        let udist = Uniform::new(0.0, 99.0);
        let mut rng = rand::thread_rng();

        for n in 0..12345 {
            if n % 17 == 0 {
                //change the delay sometimes
                delay = udist.sample(&mut rng);
                d.set_delay(delay);
            }
            let float_n = n as f64;

            let v = d.tick(float_n);

            if float_n >= delay.ceil() {
                //after the transient phase we expect values interpolated between values we previously put in
                let out_a = float_n - delay.floor();
                let out_b = float_n - delay.ceil();

                let expected_out = out_a * (1.0 - delay.fract()) + out_b * delay.fract();

                assert_relative_eq!(v, expected_out);
            } else if float_n <= delay.floor() {
                //At the beginning we should see zeroes
                assert_relative_eq!(v, 0.0);
            }
            //In between these two cases we are interpolating between initial values and values we entered.
            //not checking these for now.q
        }
    }

    #[test]
    pub fn frac_max_delay1() {
        let mut d = DelayLineFracLin::new(vec![0; 100], 99.0);
        d.tick(0);
    }

    #[test]
    pub fn frac_max_delay2() {
        let mut d = DelayLineFracLin::new(vec![0; 100], 95.0);
        d.set_delay(99.0);
        d.tick(0);
    }
}
