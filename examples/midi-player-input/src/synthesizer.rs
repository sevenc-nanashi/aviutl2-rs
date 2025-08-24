use std::sync::Arc;

pub const SAMPLE_RATE: u32 = 44100;
const MASTER_VOLUME: f32 = 0.2; // Volume level of master track (0.0 to 1.0)
const VOLUME: f32 = 1.0; // Volume level (0.0 to 1.0)
const CLIP: f32 = 1.0; // Clip value for audio samples (0.0 to 1.0)

static PIANO: std::sync::LazyLock<Arc<rustysynth::SoundFont>> = std::sync::LazyLock::new(|| {
    let piano_sf2 = include_bytes!("../piano.sf2").to_vec();
    let mut piano_sf2 = std::io::Cursor::new(piano_sf2);
    Arc::new(rustysynth::SoundFont::new(&mut piano_sf2).expect("Failed to load piano soundfont"))
});

#[derive(Debug)]
pub struct Synthesizer {
    pub synthesizer: rustysynth::Synthesizer,
    pub expected_next_sample: u64,
    pub event_index: usize,

    pub track: Arc<crate::track::Track>,
}

impl Synthesizer {
    pub fn new(track: Arc<crate::track::Track>) -> anyhow::Result<Self> {
        let synthesizer = rustysynth::Synthesizer::new(
            &PIANO,
            &rustysynth::SynthesizerSettings::new(SAMPLE_RATE as i32),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create synthesizer: {}", e))?;
        Ok(Self {
            synthesizer,
            expected_next_sample: 0,
            event_index: 0,
            track,
        })
    }

    pub fn reset(&mut self) {
        self.event_index = 0;
        self.synthesizer.reset();
    }
    pub fn render(&mut self, length: i32, start_sample: u64, end_sample: u64) -> Vec<(f32, f32)> {
        let mut samples = Vec::with_capacity(length as usize);
        let mut sample_buf_l = vec![0.0f32; 1];
        let mut sample_buf_r = vec![0.0f32; 1];
        for current_sample in start_sample..end_sample {
            let current_time = current_sample as f64 / SAMPLE_RATE as f64;
            let note_activate_buffer = self.procced_to(current_time);
            self.apply_notes(note_activate_buffer);
            self.synthesizer
                .render(&mut sample_buf_l, &mut sample_buf_r);

            samples.push((
                (sample_buf_l[0] * VOLUME * MASTER_VOLUME).clamp(-CLIP, CLIP),
                (sample_buf_r[0] * VOLUME * MASTER_VOLUME).clamp(-CLIP, CLIP),
            ));
        }
        self.expected_next_sample = end_sample;
        samples
    }

    pub fn apply_notes(
        &mut self,
        note_activate_buffer: std::collections::HashMap<u8, Option<crate::track::Note>>,
    ) {
        for (&midi, note) in &note_activate_buffer {
            match note {
                Some(note) => {
                    self.synthesizer.note_on(
                        0,
                        note.midi_note as i32,
                        note.velocity as i32, // Normalize velocity
                    );
                }
                None => {
                    self.synthesizer.note_off(0, midi as i32);
                }
            }
        }
    }

    pub fn procced_to(
        &mut self,
        current_time: f64,
    ) -> std::collections::HashMap<u8, Option<crate::track::Note>> {
        let mut note_activate_buffer = std::collections::HashMap::new();
        while self.event_index < self.track.events.len() {
            let (event_time, event) = &self.track.events[self.event_index];
            if *event_time > current_time {
                break; // No more events for this sample
            }

            // Process the event
            match event {
                crate::track::NoteEvent::NoteOn(note) => {
                    match note_activate_buffer.get(&note.midi_note) {
                        Some(Some(_)) => {
                            note_activate_buffer.insert(note.midi_note, None);
                        }
                        Some(None) | None => {
                            note_activate_buffer.insert(note.midi_note, Some(note));
                        }
                    }
                }
                crate::track::NoteEvent::NoteOff(midi_note) => {
                    match note_activate_buffer.get(midi_note) {
                        Some(Some(_note)) => {
                            note_activate_buffer.remove(midi_note);
                        }
                        None | Some(None) => {
                            note_activate_buffer.insert(*midi_note, None);
                        }
                    }
                }
            }

            self.event_index += 1;
        }
        note_activate_buffer
            .iter()
            .map(|(&k, &v)| (k, v.cloned()))
            .collect()
    }
}
