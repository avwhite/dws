use crate::delay_line::DelayLineFracLin;

use dasp::{Frame, Sample};
use dasp_signal::{Noise, Signal};

pub struct PluckedString<T> {
    string_delay: DelayLineFracLin<Vec<dasp::frame::Mono<T>>>,
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
            pick_noise: dasp_signal::noise(0),
        }
    }

    pub fn note_on(&mut self, frequency: f64) {
        // Load noise into  string_delay
        let delay = 48000.0 / frequency; // @todo get sample rate from vst host somehow.
        self.string_delay.set_delay(delay);

        for _ in 0..(delay.ceil()) as usize {
            self.string_delay.tick([self.pick_noise.next().to_sample()]);
        }
    }

    pub fn tick(&mut self) -> dasp::frame::Mono<T> {
        let out = self.string_delay.tap_output().scale_amp(0.977.to_sample());
        self.string_delay.tick(out);
        return out;
    }
}
