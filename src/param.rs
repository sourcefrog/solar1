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
const OSC1_LEVEL: usize = 5;
const OSC2_TUNE: usize = 6;
const OSC2_LEVEL: usize = 7;

const N_PARAM: usize = 8;

// Scaling factors from the [0..1] range to the semantic range.
const RELEASE_SCALE: f32 = 10.0;

/// A static definition of a single parameter.
struct ParamDef {
    name: &'static str,
    label: &'static str,
    default: f32,
}

/// All the defined parameters, with indexes matching the constants above.
const PARAMS: [ParamDef; N_PARAM] = [
    ParamDef {
        name: "Attack",
        label: "s",
        default: 0.3,
    },
    ParamDef {
        name: "Decay",
        label: "s",
        default: 0.3,
    },
    ParamDef {
        name: "Sustain",
        label: "",
        default: 0.8,
    },
    ParamDef {
        name: "Release",
        label: "s",
        default: 0.1,
    },
    ParamDef {
        name: "Osc1 Tune",
        label: "",
        default: 0.56,
    },
    ParamDef {
        name: "Osc1 Level",
        label: "",
        default: 0.5,
    },
    ParamDef {
        name: "Osc2 Tune",
        label: "",
        default: 0.43,
    },
    ParamDef {
        name: "Osc2 Level",
        label: "",
        default: 0.5,
    },
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
    pub fn osc1_freq_mul(&self) -> f64 {
        frequency_multiplier(self.copy_params()[OSC1_TUNE]) as f64
    }

    pub fn osc1_level(&self) -> f64 {
        self.copy_params()[OSC1_LEVEL] as f64
    }

    pub fn osc2_freq_mul(&self) -> f64 {
        frequency_multiplier(self.copy_params()[OSC2_TUNE]) as f64
    }

    pub fn osc2_level(&self) -> f64 {
        self.copy_params()[OSC2_LEVEL] as f64
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
        let mut p = [0.0; N_PARAM];
        for (i, def) in PARAMS.iter().enumerate() {
            p[i] = def.default;
        }
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

    fn get_parameter_name(&self, index: i32) -> String {
        PARAMS[index as usize].name.to_owned()
    }

    fn get_parameter_text(&self, index: i32) -> String {
        let pval = self.get_parameter(index);
        match index as usize {
            RELEASE => format!("{:.3}", pval * RELEASE_SCALE),
            _ => format!("{:.3}", pval),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        PARAMS[index as usize].label.to_owned()
    }
}
