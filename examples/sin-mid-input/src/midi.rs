#![allow(dead_code)]
pub fn midi_note_to_freq(midi_note: u8) -> f64 {
    // Frequency of A4 (MIDI note 69) is 440 Hz
    // F(n) = 440 * 2^((n - 69)/12)
    440.0 * 2.0f64.powf((midi_note as f64 - 69.0) / 12.0)
}

#[ouroboros::self_referencing(pub_extras)]
#[derive(Debug)]
pub struct OwnedSmf {
    pub content: Vec<u8>,
    #[borrows(content)]
    #[covariant]
    pub mid: midly::Smf<'this>,
}
#[derive(Debug, Clone)]
pub struct Note {
    pub midi_note: u8,
    pub start_tick: u64,
    pub velocity: u8, // MIDI velocity for amplitude
}

#[derive(Debug, Clone)]
pub enum NoteEvent {
    NoteOn(Note),
    NoteOff(u8),
}
