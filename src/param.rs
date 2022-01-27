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

/// A scaling curve between the [0..1] range of a parameter, and the value
/// used by the synth and shown in its text.
enum Curve {
    /// Simply map 0..1 to 0..1.
    Identity,
    /// Linear mapping to a different range.
    Linear(f32, f32),
    /// Exponentially scaled value between `2**a` and `2**b`.
    Exp2(f32, f32),
}

impl Curve {
    /// Scale forward from 0..1 to the output.
    pub fn scale(&self, v: f32) -> f32 {
        use Curve::*;
        match self {
            Identity => v,
            Linear(a, b) => a + (b - a) * v,
            Exp2(a, b) => (a + (b - a) * v).exp2(),
        }
    }

    /// Reverse the transformation, to get back to a value in 0..1.
    pub fn reverse(&self, y: f32) -> f32 {
        use Curve::*;
        match self {
            Identity => y,
            Linear(a, b) => (y - a) / (b - a),
            Exp2(a, b) => (y.log2() - a) / (b - a),
        }
    }
}

/// A static definition of a single parameter.
struct ParamDef {
    name: &'static str,
    label: &'static str,
    default: f32,
    curve: Curve,
}

/// All the defined parameters, with indexes matching the constants above.
const PARAMS: [ParamDef; N_PARAM] = [
    ParamDef {
        name: "Attack",
        label: "s",
        default: 0.3,
        curve: Curve::Identity,
    },
    ParamDef {
        name: "Decay",
        label: "s",
        default: 0.3,
        curve: Curve::Identity,
    },
    ParamDef {
        name: "Sustain",
        label: "",
        default: 0.8,
        curve: Curve::Identity,
    },
    ParamDef {
        name: "Release",
        label: "s",
        default: 0.1,
        curve: Curve::Linear(0.0, 10.0),
    },
    ParamDef {
        name: "Osc1 Tune",
        label: "octave",
        default: 0.56,
        curve: Curve::Linear(-1.0, 1.0),
    },
    ParamDef {
        name: "Osc1 Level",
        label: "",
        default: 0.5,
        curve: Curve::Identity,
    },
    ParamDef {
        name: "Osc2 Tune",
        label: "octave",
        default: 0.43,
        curve: Curve::Linear(-1.0, 1.0),
    },
    ParamDef {
        name: "Osc2 Level",
        label: "",
        default: 0.5,
        curve: Curve::Identity,
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

    fn scaled_value(&self, index: usize) -> f32 {
        PARAMS[index].curve.scale(self.get_parameter(index as i32))
    }

    /// Return the frequency multiplier for osc1.
    pub fn osc1_freq_mul(&self) -> f64 {
        self.scaled_value(OSC1_TUNE).exp2() as f64
    }

    pub fn osc1_level(&self) -> f64 {
        self.copy_params()[OSC1_LEVEL] as f64
    }

    pub fn osc2_freq_mul(&self) -> f64 {
        self.scaled_value(OSC2_TUNE).exp2() as f64
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
        let def = &PARAMS[index as usize];
        let yval = def.curve.scale(pval);
        let show_sign = match def.curve {
            Curve::Linear(a, _b) if a < 0.0 => true,
            _ => false,
        };
        if show_sign {
            format!("{:+.3}", yval)
        } else {
            format!("{:.3}", yval)
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        PARAMS[index as usize].label.to_owned()
    }
}
