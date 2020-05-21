use sample::ring_buffer;
use sample::Frame;
use sample::Sample;

use crate::delay_line;

#[derive(Clone, Copy)]
pub struct EchoParameters
{
    pub attenuation: f32,
    pub length: usize
}

impl EchoParameters
{
    pub fn from_distances(d : f32, h: f32, sample_rate: usize) -> Self
    {
        let frame_t = 1.0 / sample_rate as f32;
        let r = ((h*h + (d*d/4.0)) as f32).sqrt();
        let m = ((2.0 * r - d) / (343.0*frame_t)).round() as usize;
        let g = d / (2.0 * r);

        EchoParameters
        {
            attenuation: g,
            length : m,
        }
    }
}

pub struct Echo<T>
{
    delay_line : delay_line::DelayLine<Vec<T>>,
    params : EchoParameters,
}

impl<T: Frame> Echo<T>
{
    pub fn new(d : f32, h: f32, sample_rate: usize, capacity: usize) -> Self {
        let params = EchoParameters::from_distances(d, h, sample_rate);

        assert!(params.length < capacity);

        Echo {
            delay_line : delay_line::DelayLine::new(vec![T::equilibrium(); capacity], params.length),
            params: params
        }
    }

    pub fn set_params(self: &mut Self, params: EchoParameters)
    {
        self.params = params;

        self.delay_line.set_delay(params.length);
    }

    pub fn tick(self : &mut Self, in_frame: T) -> T {
        let signed_in = in_frame.to_signed_frame();

        let out = self.delay_line.tick(in_frame);
        
        out.scale_amp(self.params.attenuation.to_sample()).add_amp(signed_in)
    }
}