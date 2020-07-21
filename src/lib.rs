pub mod delay_line;
pub mod effects;
pub mod filter;
pub mod instruments;

use std::sync::Arc;

use vst::api::Events;
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::plugin_main;
use vst::util::ParameterTransfer;

struct DwsParams {
    param_transfer: ParameterTransfer,
}

impl DwsParams {
    fn normalize_parameter(index: i32, value: f32) -> f32 {
        match index {
            0 => value / 10.0,
            1 => value / 0.01,
            2 => value,
            _ => 0.0,
        }
    }

    fn denormalize_parameter(index: i32, value: f32) -> f32 {
        match index {
            0 => value * 10.0,
            1 => value * 0.01,
            2 => value,
            _ => 0.0,
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
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "rate".to_string(),
            1 => "amount".to_string(),
            2 => "depth".to_string(),
            _ => "computer says no".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "Hz".to_string(),
            1 => "s".to_string(),
            2 => "".to_string(),
            _ => "computer says no".to_string(),
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
            params: std::sync::Arc::new(DwsParams {
                param_transfer: ParameterTransfer::new(3),
            }),
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

struct VstPluckedString {
    plucked_string: instruments::PluckedString<f32>,
}

impl Default for VstPluckedString {
    fn default() -> VstPluckedString {
        VstPluckedString {
            plucked_string: instruments::PluckedString::new(),
        }
    }
}

impl Plugin for VstPluckedString {
    fn get_info(&self) -> Info {
        Info {
            name: "pluced_string".to_string(),
            unique_id: 847123, // Used by hosts to differentiate between plugins.
            inputs: 0,
            outputs: 1,
            category: Category::Synth,

            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();

        let output = outputs.get_mut(0).into_iter();

        for o in output {
            *o = self.plucked_string.tick()[0];
        }
    }

    fn process_events(&mut self, events: &Events) {
        // Some events aren't MIDI events - so let's do a match
        // to make sure we only get MIDI, since that's all we care about.
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    // Check if it's a noteon or noteoff event.
                    // This is difficult to explain without knowing how the MIDI standard works.
                    // Basically, the first byte of data tells us if this signal is a note on event
                    // or a note off event.  You can read more about that here:
                    // https://www.midi.org/specifications/item/table-1-summary-of-midi-message
                    match ev.data[0] {
                        // if note on, increment our counter
                        144 => self.plucked_string.note_on(
                            440.0 * (1.0594630943592953 as f64).powf(ev.data[1] as f64 - 69.0),
                        ),
                        // if note off, nothing
                        128 => (),
                        _ => (),
                    }
                    // if we cared about the pitch of the note, it's stored in `ev.data[1]`.
                }
                // We don't care if we get any other type of event
                _ => (),
            }
        }
    }
}

plugin_main!(VstPluckedString);
