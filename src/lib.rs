// Copyright 2022 Martin Pool

//! Solar1 experimental drone sawtooth synth.
//!
//! Based on the vst-rs `sine_synth` example and inspired by the Solar 50.

mod adsr;
mod midi;
mod param;

use std::f64::consts::PI;
use std::sync::Arc;

use log::*;
use simple_logger::SimpleLogger;

use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};

use crate::adsr::{AdsrEnvelope, AdsrParams};
use crate::midi::MidiNote;
use crate::param::Params;

pub const TAU: f64 = PI * 2.0;

struct Solar1 {
    sample_rate: f64,
    time: f64,
    note: Option<MidiNote>,
    envelope: AdsrEnvelope,
    parameters: Arc<Params>,
}

impl Solar1 {
    fn time_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }

    /// Process an incoming midi event.
    ///
    /// The midi data is split up like so:
    ///
    /// `data[0]`: Contains the status and the channel. Source: [source]
    /// `data[1]`: Contains the supplemental data for the message - so, if this was a NoteOn then
    ///            this would contain the note.
    /// `data[2]`: Further supplemental data. Would be velocity in the case of a NoteOn message.
    ///
    /// [source]: http://www.midimountain.com/midi/midi_status.htm
    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(MidiNote(data[1])),
            144 => self.note_on(MidiNote(data[1])),
            _ => (),
        }
    }

    fn note_on(&mut self, note: MidiNote) {
        // TODO: Keep a set of active notes and play with polyphony.
        let adsr_params = self.parameters.adsr();
        info!("note_on {note:?} {adsr_params:?}");
        self.envelope = adsr::AdsrEnvelope::new(adsr_params);
        self.envelope.trigger(self.time);
        self.note = Some(note)
    }

    fn note_off(&mut self, note: MidiNote) {
        // info!("note_off {note:?}");
        if self.note == Some(note) {
            // This was the most recently played note?
            self.envelope.release(self.time);
        }
        // Don't forget the note; let it ring out.
    }
}

impl Plugin for Solar1 {
    fn new(_host: HostCallback) -> Self {
        let _ = SimpleLogger::new().init(); // It might be already initialized; we don't care.

        let parameters = Params::default();
        let envelope = adsr::AdsrEnvelope::new(parameters.adsr());

        info!("Solar1 created!");
        Solar1 {
            sample_rate: 44100.0,
            time: 0.0,
            note: None,
            envelope,
            parameters: Arc::new(parameters),
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Solar1".to_string(),
            vendor: "Martin Pool".to_string(),
            unique_id: 484940,
            category: Category::Synth,
            inputs: 0, // No audio inputs
            outputs: 2,
            parameters: Params::len() as i32,
            initial_delay: 0,
            ..Info::default()
        }
    }

    #[allow(clippy::single_match)]
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                // More events can be handled here.
                _ => (),
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = f64::from(rate);
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = self.time_per_sample();
        let mut output_sample;
        for sample_idx in 0..samples {
            let time = self.time;
            if let Some(current_note) = &self.note {
                let base_freq = current_note.frequency();
                // What position are we at in this cycle?
                let signal0 = (time * base_freq) % 1.0 - 0.5;

                let signal1 = (time * base_freq * self.parameters.osc1_freq_mul()) % 1.0 - 0.5;

                let signal2 = (time * base_freq * self.parameters.osc2_freq_mul()) % 1.0 - 0.5;

                let signal = signal0
                    + signal1 * self.parameters.osc1_level()
                    + signal2 * self.parameters.osc2_level();

                let alpha = self.envelope.sample(time);

                output_sample = (signal * alpha) as f32;

                self.time += per_sample;
            } else {
                output_sample = 0.0;
            }
            // Output this value in unison across probably two stereo output channels.
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe,
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn vst::plugin::PluginParameters> {
        self.parameters.clone()
    }
}

vst::plugin_main!(Solar1);
