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

pub struct FlangeParameters
{
    frame_time: f64,
    rate: f64,
    amount: f64,
    depth: f64
}

pub struct Flange<T>
{
    params : FlangeParameters,
    delay_line: delay_line::DelayLineFracLin<Vec<T>>,
    time: f64,
}

impl<T: Frame> Flange<T>
{
    pub fn new(rate: f64, amount: f64, depth: f64, sample_rate: usize) -> Self {
        Flange {
            params: FlangeParameters {
                frame_time : 1.0 / sample_rate as f64,
                rate: rate,
                amount: amount * sample_rate as f64,
                depth: depth,
            },
            delay_line: delay_line::DelayLineFracLin::new(vec![T::equilibrium(); 100000], T::equilibrium(), 1000.0),
            time: 0.0,
        }
    }

    pub fn tick(self: &mut Self, in_frame: T) -> T {
        self.time += self.params.frame_time;
        let sine = (self.params.rate * self.time * 2.0 * std::f64::consts::PI).sin(); //Would a look up table give better performance?
        self.delay_line.set_delay(self.params.amount * sine + self.params.amount + 0.0005);
        let a = self.delay_line.tick(in_frame);
        let out = a.scale_amp(self.params.depth.to_sample()).add_amp(in_frame.to_signed_frame());

        return out;
    }
}