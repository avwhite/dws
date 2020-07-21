use crate::delay_line::DelayLineFracLin;
use crate::filter::FIRFilter;

use dasp::frame::Mono;
use dasp::{Frame, Sample};
use dasp_signal::{Noise, Signal};

pub struct PluckedString<T> {
    string_delay: DelayLineFracLin<Vec<Mono<T>>>,
    string_filter: FIRFilter<Mono<T>>,
    pick_noise: Noise,
}

impl<T> PluckedString<T>
where
    T: dasp::Sample,
    T: dasp::sample::FromSample<f64>,
{
    pub fn new() -> PluckedString<T> {
        //TODO calculate length based on min/max frequency. maybe random seed?
        PluckedString {
            string_delay: DelayLineFracLin::new(
                vec![dasp::frame::Mono::<T>::EQUILIBRIUM; 100000],
                109.09,
            ),
            string_filter: FIRFilter::new(vec![0.5, 0.5]),
            pick_noise: dasp_signal::noise(0),
        }
    }

    pub fn note_on(&mut self, frequency: f64) {
        // Load noise into  string_delay
        // @todo get sample rate from vst host somehow.
        // @todo minus 0.5 to compensate for string filter delay. not sure if this is the correct way to do this.
        let delay = (48000.0 / frequency) - 0.5;
        self.string_delay.set_delay(delay);

        for _ in 0..(delay.ceil()) as usize {
            self.string_delay.tick([self.pick_noise.next().to_sample()]);
        }
    }

    pub fn tick(&mut self) -> dasp::frame::Mono<T> {
        let out = self.string_filter.tick(self.string_delay.tap_output());
        self.string_delay.tick(out);
        return out;
    }
}
