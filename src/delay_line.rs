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
    pub fn capacitiy(&self) -> usize {
        self.data.slice().len()
    }

    pub fn tick(&mut self, item: S::Element) -> S::Element
    where
        S: SliceMut,
    {
        let out = self.data.slice_mut()[self.out_point];
        self.out_point = (self.out_point + 1).rem_euclid(self.capacitiy());

        self.data.slice_mut()[self.in_point] = item;
        self.in_point = (self.in_point + 1).rem_euclid(self.capacitiy());

        return out;
    }

    /// Borrows the item at the given index relative to the input
    pub fn tap(&self, index: usize) -> S::Element {
        assert!(index + 1 < self.capacitiy());

        let wrapped_index: usize;

        if index + 1 > self.in_point {
            wrapped_index = self.capacitiy() - (index + 1 - self.in_point);
        }
        else {
            wrapped_index = self.in_point - (index + 1);
        }

        self.data.slice()[wrapped_index]
    }

    pub fn set_delay(&mut self, delay: usize)
    {
        if delay > self.in_point {
            self.out_point = self.capacitiy() - (delay - self.in_point);
        }
        else {
            self.out_point = self.in_point - delay;
        }
    }

    pub fn new(data: S, delay: usize) -> Self {
        assert!(delay < data.slice().len());

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
    fractional_frame: T::Element,
    fractional_delay_part: f64,
}

impl<T> DelayLineFracLin<T>
where
    T : Slice,
    T::Element : Frame
{
    pub fn new(data: T, fractional_frame : T::Element, delay: f64) -> Self {
        assert!(data.slice().len() > 0);

        let integer_part = delay.trunc() as usize;
        let fractional_part = delay.fract();

        DelayLineFracLin {
            delay_line: DelayLine::new(data, integer_part),
            fractional_frame: fractional_frame,
            fractional_delay_part : fractional_part
        }
    }

    pub fn tick(&mut self, item: T::Element) -> T::Element
    where
        T: SliceMut
    {
        let out_integer = self.delay_line.tick(item);
        let out_integer_part = out_integer.scale_amp((1.0 - self.fractional_delay_part).to_sample());
        let out_frac_part = self.fractional_frame.scale_amp(self.fractional_delay_part.to_sample());

        out_integer_part.add_amp(out_frac_part.to_signed_frame())
    }

    pub fn set_delay(&mut self, delay: f64)
    {

        let integer_part = delay.trunc() as usize;
        let fractional_part = delay.fract();

        self.delay_line.set_delay(integer_part);
        self.fractional_delay_part = fractional_part;

        self.fractional_frame = self.delay_line.tap(integer_part + 1);
    }
}