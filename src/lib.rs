pub mod delay_line;
pub mod effects;

use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::util::ParameterTransfer;
use vst::plugin::{Info, Plugin, PluginParameters};
use vst::plugin_main;


struct DwsParams {
    param_transfer: ParameterTransfer
}

impl DwsParams {
    fn normalize_parameter(index: i32, value: f32) -> f32 {
        match index {
            0 => value / 10.0,
            1 => value / 0.01,
            2 => value,
            _ => 0.0
        }
    }

    fn denormalize_parameter(index: i32, value: f32) -> f32 {
        match index {
            0 => value * 10.0,
            1 => value * 0.01,
            2 => value,
            _ => 0.0
        }
    }

    fn get_denorm_parameter(&self, index: i32) -> f32 {
        self.param_transfer.get_parameter(index as usize)
    }

    fn set_denorm_parameter(&self, index: i32, value: f32) {
        self.param_transfer.set_parameter(index as usize, value);
    }
}

impl PluginParameters for DwsParams {
    fn get_parameter_name(&self, index: i32) -> String{
        match index {
            0 => "rate".to_string(),
            1 => "amount".to_string(),
            2 => "depth".to_string(),
            _ => "computer says no".to_string()
        }
    }

    fn get_parameter_label(&self, index: i32) -> String{
        match index {
            0 => "Hz".to_string(),
            1 => "s".to_string(),
            2 => "".to_string(),
            _ => "computer says no".to_string()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        DwsParams::normalize_parameter(index, self.get_denorm_parameter(index))
    }

    fn get_parameter_text(&self, index: i32) -> String {
        format!("{number:.5}", number = self.get_denorm_parameter(index))
    }

    fn set_parameter(&self, index: i32, value: f32) {
        self.set_denorm_parameter(index, DwsParams::denormalize_parameter(index, value));
    }
}

struct Dws {
    flange: effects::Flange<dasp::frame::Stereo<f32>>,
    params: Arc<DwsParams>,
}

impl Default for Dws {
    fn default() -> Dws {
        Dws {
            flange: effects::Flange::new(5.0, 0.001, 0.3, 48000),
            params: std::sync::Arc::new( DwsParams { param_transfer: ParameterTransfer::new(3) } ),
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
            parameters: 3,

            ..Default::default()
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.params.clone()
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (index, value) in self.params.param_transfer.iterate(true) {
            match index {
                0 => self.flange.set_rate(value as f64),
                1 => self.flange.set_amount(value as f64), 
                2 => self.flange.set_depth(value as f64), 
                _ => {}
            }
        }

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
