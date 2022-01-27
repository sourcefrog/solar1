#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MidiNote(pub u8);

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
