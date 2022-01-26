// Copyright 2022 Martin Pool

//! Solar1 experimental drone sawtooth synth.
//!
//! Based on the vst-rs `sine_synth` example and inspired by the Solar 50.

use log::*;
use simple_logger::SimpleLogger;

use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{CanDo, Category, HostCallback, Info, Plugin};

use std::f64::consts::PI;

mod adsr;
use adsr::{AdsrEnvelope, AdsrParams};

pub const TAU: f64 = PI * 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiNote(u8);

impl MidiNote {
    /// Convert the midi note's pitch into the equivalent frequency.
    ///
    /// This function assumes A4 is 440hz.
    pub fn frequency(&self) -> f64 {
        const A4_PITCH: i8 = 69;
        const A4_FREQ: f64 = 440.0;

        // Midi notes can be 0-127
        ((f64::from(self.0 as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
    }
}

struct Solar1 {
    sample_rate: f64,
    time: f64,
    note: Option<MidiNote>,
    envelope: AdsrEnvelope,
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
        // info!("note_on {note:?}");
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

        let adsr_params = AdsrParams {
            attack_s: 0.2,
            decay_s: 0.5,
            sustain_level: 0.5,
            release_s: 1.0,
        };
        let envelope = adsr::AdsrEnvelope::new(adsr_params);

        info!("Solar1 created!");
        Solar1 {
            sample_rate: 44100.0,
            time: 0.0,
            note: None,
            envelope,
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "Solar1".to_string(),
            vendor: "Martin Pool".to_string(),
            unique_id: 484940,
            category: Category::Synth,
            inputs: 2,
            outputs: 2,
            parameters: 0,
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
                // What position are we at in this cycle?
                let cycle_len = 1.0 / current_note.frequency();
                let signal = (time % cycle_len) / cycle_len - 0.5;

                let cycle2_len = cycle_len * 1.3;
                let signal2 = (time % cycle2_len) / cycle2_len - 0.5;

                let signal = signal * 0.8 + signal2 * 0.4;

                // let phase = (time % cycle_len) / cycle_len;
                // let signal = if phase < 0.5 { -1.0 } else { 1.0 };

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
}

vst::plugin_main!(Solar1);
