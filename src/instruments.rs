use crate::delay_line::DelayLineFracLin;
use crate::filter::FIRFilter;

use dasp::frame::Mono;
use dasp::{Frame, Sample};
use dasp_signal::{Noise, Signal};

pub struct PluckedString<T> {
    string_delay: DelayLineFracLin<Vec<Mono<T>>>,
    string_filter: FIRFilter<Mono<T>>,
    pick_noise: Noise,
    pub brightness: f64,
    pub sustain: f64,
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
            string_filter: FIRFilter::new(vec![0.0; 3]),
            pick_noise: dasp_signal::noise(0),
            sustain: 1.0,
            brightness: 0.1,
        }
    }

    pub fn note_on(&mut self, frequency: f64) {
        // Load noise into  string_delay
        // @todo get sample rate from vst host somehow.
        // @todo minus 0.5 to compensate for string filter delay. not sure if this is the correct way to do this.
        let period = 1.0 / frequency;
        let delay = 48000.0 * period;
        // - 1.0 to compensate for delay introduced by filter
        self.string_delay.set_delay(delay - 1.0);

        // See PASP §9.1.2
        let rho = (-6.91 * period / self.sustain).exp();
        let g0 = rho * (1.0 + self.brightness) / 2.0;
        let g1 = rho * (1.0 - self.brightness) / 4.0;

        let filter_coefs = self.string_filter.get_mut_coefficients();
        filter_coefs[0] = g1;
        filter_coefs[1] = g0;
        filter_coefs[2] = g1;

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
