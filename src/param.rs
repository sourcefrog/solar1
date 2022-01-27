use std::cell::Cell;
use std::sync::Mutex;

use vst::plugin::PluginParameters;

const N_PARAM: usize = 8;

/// Plugin parameters: these map into knobs or sliders in the DAW.
#[derive(Default, Debug)]
pub struct Params {
    /// The internal form of the params as an indexed array of f32.
    p: Mutex<Cell<[f32; N_PARAM]>>,
}

impl Params {
    /// Return the number of parameters.
    pub fn len() -> usize {
        N_PARAM
    }

    /// Return the frequency multiplier for osc1.
    pub fn osc1_freq_mul(&self) -> f32 {
        // TODO: Scale this so that 0.0 is half the frequency and 1.0 is double?
        frequency_multiplier(self.get_parameter(0))
    }
}

/// Scale a parameter in the range 0 to 1, to a frequency multiplier between 0.5 and 2.0.
fn frequency_multiplier(a: f32) -> f32 {
    (a * 2.0 - 1.0).exp2()
}

impl PluginParameters for Params {
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Osc 1 Tune".into(),
            _ => format!("Param {index}"),
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        // This copies out all the parameters, which is OK and avoids locking.
        let p = self.p.lock().unwrap().get();
        *p.get(index as usize).unwrap_or(&0.0)
    }

    fn set_parameter(&self, index: i32, value: f32) {
        let plock = self.p.lock().unwrap();
        let mut pcopy = plock.get();
        pcopy.get_mut(index as usize).map(|pv| *pv = value);
        plock.set(pcopy);
    }
}
