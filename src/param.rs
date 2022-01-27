use std::cell::Cell;
use std::sync::Mutex;

use vst::plugin::PluginParameters;

use crate::AdsrParams;

// Parameter assignments
const ATTACK: usize = 0;
const DECAY: usize = 1;
const SUSTAIN: usize = 2;
const RELEASE: usize = 3;
const OSC1_TUNE: usize = 4;

const N_PARAM: usize = 6;

// Scaling factors from the [0..1] range to the semantic range.
const RELEASE_SCALE: f32 = 10.0;

const PARAM_NAMES: [&'static str; N_PARAM] = [
    "Attack",
    "Decay",
    "Sustain",
    "Release",
    "Osc1 Tune",
    "Osc1 Level",
];

/// Plugin parameters: these map into knobs or sliders in the DAW.
#[derive(Debug)]
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
        frequency_multiplier(self.copy_params()[OSC1_TUNE])
    }

    /// Return global ADSR parameters.
    pub fn adsr(&self) -> AdsrParams {
        let p = self.copy_params();
        AdsrParams {
            attack_s: p[ATTACK] as f64,
            decay_s: p[DECAY] as f64,
            sustain_level: p[SUSTAIN] as f64,
            release_s: (p[RELEASE] * RELEASE_SCALE) as f64,
        }
    }

    fn copy_params(&self) -> [f32; N_PARAM] {
        self.p.lock().unwrap().get()
    }
}

impl Default for Params {
    fn default() -> Params {
        let p = [0.3, 0.3, 0.8, 0.1, 0.56, 0.0];
        Params {
            p: Mutex::new(Cell::new(p)),
        }
    }
}

/// Scale a parameter in the range 0 to 1, to a frequency multiplier between 0.5 and 2.0.
fn frequency_multiplier(a: f32) -> f32 {
    (a * 2.0 - 1.0).exp2()
}

impl PluginParameters for Params {
    fn get_parameter_name(&self, index: i32) -> String {
        PARAM_NAMES
            .get(index as usize)
            .map(|&s| s.into())
            .unwrap_or_else(|| format!("Param {index}"))
    }

    fn get_parameter(&self, index: i32) -> f32 {
        // This copies out all the parameters, which is OK and avoids locking.
        let p = self.copy_params();
        *p.get(index as usize).unwrap_or(&0.0)
    }

    fn set_parameter(&self, index: i32, value: f32) {
        let plock = self.p.lock().unwrap();
        let mut pcopy = plock.get();
        pcopy.get_mut(index as usize).map(|pv| *pv = value);
        plock.set(pcopy);
    }

    fn get_parameter_text(&self, index: i32) -> String {
        let pval = self.get_parameter(index);
        match index as usize {
            RELEASE => format!("{:.3}", pval * RELEASE_SCALE),
            _ => format!("{:.3}", pval),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index as usize {
            ATTACK | DECAY | RELEASE => "s",
            _ => "",
        }
        .into()
    }
}
