pub mod delay_line;
pub mod effects;

use vst::buffer::AudioBuffer;
use vst::plugin::{Info, Plugin};
use vst::plugin_main;

struct Dws {
    flange: effects::Flange<dasp::frame::Stereo<f32>>,
}

impl Default for Dws {
    fn default() -> Dws {
        Dws {
            flange: effects::Flange::new(5.0, 0.001, 0.3, 48000),
        }
    }
}

impl Plugin for Dws {
    fn get_info(&self) -> Info {
        Info {
            name: "dws".to_string(),
            unique_id: 84781384, // Used by hosts to differentiate between plugins.
            inputs: 2,
            outputs: 2,

            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (inputs, mut outputs) = buffer.split();

        let left_in = inputs.get(0).into_iter();
        let right_in = inputs.get(1).into_iter();

        let left_out = outputs.get_mut(0).into_iter();
        let right_out = outputs.get_mut(1).into_iter();

        for ((li, ri), (lo, ro)) in left_in.zip(right_in).zip(left_out.zip(right_out)) {
            let o = self.flange.tick([*li, *ri]);
            *lo = o[0];
            *ro = o[1];
        }
    }
}

plugin_main!(Dws); // Important!
