pub mod delay_line;
pub mod effects;

#[macro_use]
extern crate vst;

use vst::plugin::{Info, Plugin};

#[derive(Default)]
struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "dws".to_string(),
            unique_id: 84781384, // Used by hosts to differentiate between plugins.

            ..Default::default()
        }
    }
}

plugin_main!(BasicPlugin); // Important!