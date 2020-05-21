use sample::ring_buffer::*;

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
    #[inline]
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

    /// Creates a `Fixed` ring buffer from its starting index and data buffer type.
    ///
    /// **Panic!**s if the given index is out of range of the given data slice.
    ///
    /// **Note:** This method should only be necessary if you require specifying a first index.
    /// Please see the `ring_buffer::Fixed::from` function for a simpler constructor that does not
    /// require a `first` index.
    #[inline]
    pub fn new(data: S, delay: usize) -> Self {
        assert!(delay < data.slice().len());

        DelayLine { 
            in_point: 0,
            out_point: (data.slice().len() - delay) % data.slice().len(),
            data: data,
         }
    }
}
