use crate::delay_line::DelayLine;
use dasp::Sample;

pub struct FIRFilter<F> {
    memory: DelayLine<Vec<F>>,
    coefficients: Vec<f64>,
}

impl<F> FIRFilter<F>
where
    F: dasp::Frame,
{
    pub fn new(coefficients: Vec<f64>) -> FIRFilter<F> {
        FIRFilter {
            memory: DelayLine::new(
                vec![F::EQUILIBRIUM; coefficients.len()],
                coefficients.len() - 1,
            ),
            coefficients: coefficients,
        }
    }

    pub fn get_mut_coefficients(&mut self) -> &mut [f64] {
        self.coefficients.as_mut_slice()
    }

    pub fn get_coefficients(&self) -> &[f64] {
        self.coefficients.as_slice()
    }

    pub fn tick(&mut self, input: F) -> F {
        let mut output = input.scale_amp(self.coefficients[0].to_sample());
        for i in 1..self.coefficients.len() {
            //output += tap(i) * coefficients[i] but for general frames
            output = output.add_amp(
                self.memory
                    .tap(i - 1)
                    .scale_amp(self.coefficients[i].to_sample())
                    .to_signed_frame(),
            );
        }
        self.memory.tick(input);
        return output;
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::*;
    #[test]
    pub fn fir_impulse_response() {
        //For a fir filter the impules response should equal the coefficients.

        let coefs = vec![0.1, 0.2, 0.0, 2.1];
        let mut filter = FIRFilter::<f64>::new(coefs.clone());

        let mut impulse = vec![0.0; coefs.len()];
        impulse[0] = 1.0;
        for (c, i) in coefs.iter().zip(impulse) {
            let o = filter.tick(i);
            assert_eq!(*c, o);
        }
    }
}
