use sample::ring_buffer;
use sample::Frame;
use sample::frame::Stereo;

pub struct Echo
{
    delay_line : ring_buffer::Fixed<Vec<Stereo<f32>>>,
    attenuation : f32
}

impl Echo
{
    pub fn new(d : f32, h: f32, sample_rate: usize) -> Echo {
        let frame_t = 1.0 / sample_rate as f32;
        let r = ((h*h + (d*d/4.0)) as f32).sqrt();
        let m = ((2.0 * r - d) / (343.0*frame_t)).round() as i64;
        let g = d / (2.0 * r);

        Echo {
            delay_line : ring_buffer::Fixed::from(vec![Stereo::<f32>::equilibrium(); m as usize]),
            attenuation : g,
        }
    }

    pub fn tick(self : &mut Self, in_frame: Stereo<f32>) -> Stereo<f32> {
        self.delay_line.push(in_frame).scale_amp(self.attenuation).add_amp(in_frame)
    }
}