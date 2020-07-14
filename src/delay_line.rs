use sample::ring_buffer::*;
use sample::frame::Frame;
use sample::Sample;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DelayLine<S> {
    in_point: usize,
    out_point: usize,
    data: S,
}

impl<S> DelayLine<S>
where
    S: Slice,
    S::Element : Copy
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
        assert!(index + 1 < self.capacity());

        let wrapped_index: usize;

        if index + 1 > self.in_point {
            wrapped_index = self.data.slice().len() - (index + 1 - self.in_point);
        }
        else {
            wrapped_index = self.in_point - (index + 1);
        }

        self.data.slice()[wrapped_index]
    }

    pub fn get_delay(&self) -> usize {
        if self.in_point >= self.out_point {
            return self.in_point - self.out_point;
        }
        else {
            return self.data.slice().len() - (self.out_point - self.in_point);
        }
    }

    /// Borrows the item at the given index relative to the output (0 is the next value to be output)
    /// we should have tap_output(n) == tap_input(delay + n)1
    pub fn tap_output(&self, index: usize) -> S::Element {
        self.tap(self.get_delay() + index)
    }

    pub fn set_delay(&mut self, delay: usize)
    {
        assert!(delay < self.capacity());

        if delay > self.in_point {
            self.out_point = self.data.slice().len() - (delay - self.in_point);
        }
        else {
            self.out_point = self.in_point - delay;
        }
    }

    pub fn new(data: S, delay: usize) -> Self {
        assert!(data.slice().len() > 1);
        assert!(delay < data.slice().len() - 1);

        DelayLine { 
            in_point: 0,
            out_point: (data.slice().len() - delay) % data.slice().len(),
            data: data,
         }
    }
}

pub struct DelayLineFracLin<T>
where
    T : Slice
{
    delay_line: DelayLine<T>,
    fractional_delay_part: f64,
}

impl<T> DelayLineFracLin<T>
where
    T : Slice,
    T::Element : Frame
{
    pub fn new(data: T, delay: f64) -> Self {
        assert!(data.slice().len() > 0);

        let integer_part = delay.trunc() as usize;
        let fractional_part = delay.fract();

        DelayLineFracLin {
            delay_line: DelayLine::new(data, integer_part),
            fractional_delay_part : fractional_part
        }
    }

    pub fn tick(&mut self, item: T::Element) -> T::Element
    where
        T: SliceMut
    {
        let out_integer = self.delay_line.tick(item);
        let out_integer_part = out_integer.scale_amp((1.0 - self.fractional_delay_part).to_sample());

        let out_frac = self.delay_line.tap_output(1);
        let out_frac_part = out_frac.scale_amp(self.fractional_delay_part.to_sample());

        out_integer_part.add_amp(out_frac_part.to_signed_frame())
    }

    pub fn set_delay(&mut self, delay: f64)
    {

        let integer_part = delay.trunc() as usize;
        let fractional_part = delay.fract();

        self.delay_line.set_delay(integer_part);
        self.fractional_delay_part = fractional_part;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    pub fn zero_integer_delay()
    {
        let mut d = DelayLine::new(vec![0; 100], 0);

        for n in 0..1000
        {
            let v = d.tick(n);

            assert_eq!(v, n);
        }
    }

    #[test]
    pub fn fixed_integer_delay()
    {
        let delay = 5;

        let mut d = DelayLine::new(vec![0; 100], delay);

        for n in 0..1000
        {
            let v = d.tick(n);

            if n >= delay
            {
                //after the transient phase we expect values we previously put in
                assert_eq!(v, n - delay);
            }
            else
            {
                //before it we expect 0, which is what the delay line is initialized with
                assert_eq!(v, 0);
            }
        }
    }
}