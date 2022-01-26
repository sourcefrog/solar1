#[derive(Debug)]
pub struct AdsrParams {
    pub attack_s: f64,
    pub decay_s: f64,
    pub sustain_level: f64,
    pub release_s: f64,
}

#[derive(Debug)]
enum AdsrEnvelopeState {
    Attack { attack_start: f64 },
    Decay { decay_start: f64 },
    Sustain,
    Release { release_start: f64 },
    Silent,
}
use AdsrEnvelopeState::*;

pub struct AdsrEnvelope {
    params: AdsrParams,
    state: AdsrEnvelopeState,
}

impl AdsrEnvelope {
    pub fn new(params: AdsrParams) -> AdsrEnvelope {
        AdsrEnvelope {
            params,
            state: Silent,
        }
    }

    pub fn trigger(&mut self, time: f64) {
        self.state = AdsrEnvelopeState::Attack { attack_start: time };
    }

    pub fn release(&mut self, time: f64) {
        match &self.state {
            Attack { .. } | Decay { .. } | Sustain => {
                self.state = Release {
                    release_start: time,
                }
            }
            Silent | Release { .. } => (),
        }
    }

    // TODO: Move to a `Signal` trait or something.
    pub fn sample(&mut self, time: f64) -> f64 {
        loop {
            match &self.state {
                Silent => return 0.0,
                Sustain => return self.params.sustain_level,
                Attack { attack_start } => {
                    let reltime = time - attack_start;
                    if reltime < 0.0 {
                        return 0.0;
                    } else if reltime > self.params.attack_s {
                        self.state = Decay {
                            decay_start: attack_start + self.params.attack_s,
                        };
                    } else {
                        return reltime / self.params.attack_s;
                    }
                }
                Decay { decay_start } => {
                    let reltime = time - decay_start;
                    if reltime > self.params.decay_s || reltime < 0.0 {
                        self.state = Sustain;
                    } else {
                        return reltime / self.params.decay_s;
                    }
                }
                Release { release_start } => {
                    let reltime = time - release_start;
                    if reltime < 0.0 || reltime > self.params.release_s {
                        self.state = Silent;
                    } else {
                        return reltime / self.params.release_s;
                    }
                }
            }
        }
    }
}