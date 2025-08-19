mod midi;
use aviutl2::input::InputPlugin;
use aviutl2::input::IntoAudio;
use itertools::Itertools;
use ordered_float::OrderedFloat;

const SAMPLE_RATE: u32 = 44100;
const TAIL_LENGTH: f64 = 1.0; // 1 second tail length

const PI_2: f32 = 2.0 * std::f32::consts::PI;

struct SinMidPlayerPlugin {}

#[derive(Debug)]
struct SinMidPlayerHandle {
    smf: midi::OwnedSmf,
    ticks_per_beat: u64,
    tempo_index_cache:
        std::sync::OnceLock<std::collections::BTreeMap<OrderedFloat<f64>, TempoIndexCacheEntry>>,
    expected_next_sample: u64,
    active_notes: std::collections::HashMap<u8, midi::Note>,
    current_track_events: Vec<(u64, midi::NoteEvent)>,
    current_track_event_index: usize,
}
#[derive(Debug, Clone)]
struct TempoIndexCacheEntry {
    ticks: u64,
    uspb: u64,
}

impl SinMidPlayerHandle {
    fn open(content: Vec<u8>) -> anyhow::Result<Self> {
        let smf = midi::OwnedSmfTryBuilder {
            content,
            mid_builder: |content| midly::Smf::parse(content),
        }
        .try_build()
        .expect("Failed to parse MIDI content");

        let ticks_per_beat = match smf.borrow_mid().header.timing {
            midly::Timing::Metrical(tpb) => tpb.as_int() as u64,
            _ => {
                return Err(anyhow::anyhow!("Only Metrical timing is supported"));
            }
        };

        Ok(SinMidPlayerHandle {
            smf,
            ticks_per_beat,
            tempo_index_cache: std::sync::OnceLock::new(),
            expected_next_sample: 0,
            active_notes: std::collections::HashMap::new(),
            current_track_events: vec![],
            current_track_event_index: 0,
        })
    }

    fn tempo_index(&self) -> &std::collections::BTreeMap<OrderedFloat<f64>, TempoIndexCacheEntry> {
        self.tempo_index_cache.get_or_init(|| {
            let mid = self.smf.borrow_mid();
            let tempo_changes = {
                let current_uspb = 60000000u64 / 120; // Default to 120 BPM
                let mut current_tick = 0u64;
                let mut tempo_changes = vec![];
                for track in &mid.tracks {
                    for event in track.iter() {
                        current_tick += event.delta.as_int() as u64;
                        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(uspb)) =
                            &event.kind
                        {
                            tempo_changes.push((current_tick, uspb.as_int() as u64));
                        }
                    }
                }
                tempo_changes.sort_by_key(|(tick, _)| *tick);
                if tempo_changes.first().is_none_or(|(tick, _)| *tick != 0) {
                    // 0tick目にテンポ変更がない場合、デフォルトのテンポを追加
                    tempo_changes.push((0, current_uspb));
                }

                tempo_changes
            };
            let mut tempo_index = std::collections::BTreeMap::new();
            tempo_index.insert(
                OrderedFloat(0.0),
                TempoIndexCacheEntry {
                    ticks: 0,
                    uspb: tempo_changes
                        .first()
                        .expect("unreachable: tempo_changes should not be empty")
                        .1,
                },
            );

            let mut current_time = 0f64;
            for ((p_ticks, p_uspb), (ticks, uspb)) in tempo_changes.iter().tuple_windows() {
                let delta_ticks = ticks - p_ticks;
                let delta_time =
                    ((*p_uspb as f64) / 1_000_000.0) * (delta_ticks / self.ticks_per_beat) as f64;
                current_time += delta_time;
                tempo_index.insert(
                    OrderedFloat(current_time),
                    TempoIndexCacheEntry {
                        ticks: *ticks,
                        uspb: *uspb,
                    },
                );
            }

            tempo_index
        })
    }
    fn ticks_to_time(&self, ticks: u64) -> f64 {
        let tempo_index = self.tempo_index();
        let mut prev = tempo_index.range(..OrderedFloat(ticks as f64)).next_back();
        if prev.is_none() {
            prev = tempo_index.first_key_value();
        }
        let (prev_time, prev_entry) = prev.expect("unreachable: tempo_index should not be empty");
        let next = tempo_index.range(OrderedFloat(ticks as f64)..).next();
        let (_next_time, next_entry) = next.unwrap_or_else(|| {
            tempo_index
                .last_key_value()
                .expect("unreachable: tempo_index should not be empty")
        });

        let delta_ticks = ticks - prev_entry.ticks;
        let delta_time = (next_entry.uspb as f64 / 1_000_000.0)
            * (delta_ticks as f64 / self.ticks_per_beat as f64);
        **prev_time + delta_time
    }
}

impl InputPlugin for SinMidPlayerPlugin {
    type InputHandle = SinMidPlayerHandle;

    fn new() -> Self {
        SinMidPlayerPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "SinMid Player Plugin".to_string(),
            input_type: aviutl2::input::InputType::Audio,
            file_filters: vec![aviutl2::FileFilter {
                name: "MIDI Files".to_string(),
                extensions: vec!["mid".to_string()],
            }],
            information: format!(
                "Sine Wave Mid Player for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/sin-mid-input",
                version = env!("CARGO_PKG_VERSION")
            ),
            concurrent: false,
            can_config: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> aviutl2::AnyResult<Self::InputHandle> {
        let content =
            std::fs::read(file).map_err(|e| anyhow::anyhow!("Failed to read MIDI file: {}", e))?;
        let handle = SinMidPlayerHandle::open(content)
            .map_err(|e| anyhow::anyhow!("Failed to open MIDI file: {}", e))?;

        aviutl2::odbg!(handle.tempo_index());

        Ok(handle)
    }

    fn get_input_info(
        &self,
        handle: &Self::InputHandle,
    ) -> anyhow::Result<aviutl2::input::InputInfo> {
        let mid = handle.smf.borrow_mid();
        let num_tracks = mid.tracks.len() as u32;
        let last_ticks = mid
            .tracks
            .iter()
            .map(|t| t.iter().map(|e| e.delta.as_int() as u64).sum::<u64>())
            .max()
            .unwrap_or(0);

        Ok(aviutl2::input::InputInfo {
            video: None,
            audio: Some(aviutl2::input::AudioInputInfo {
                num_tracks,
                sample_rate: SAMPLE_RATE,
                channels: 1, // Mono output
                num_samples: ((handle.ticks_to_time(last_ticks) + TAIL_LENGTH).max(0.0)
                    * SAMPLE_RATE as f64) as u32,
                format: aviutl2::input::AudioFormat::IeeeFloat32,
            }),
        })
    }

    fn read_audio_mut(
        &self,
        handle: &mut Self::InputHandle,
        start: i32,
        length: i32,
        _track: u32,
    ) -> anyhow::Result<impl aviutl2::input::IntoAudio> {
        let start_sample = start as u64;
        let end_sample = start_sample + length as u64;
        if start_sample != handle.expected_next_sample {
            // 再生位置が変更されたので、再構築
            handle.current_track_event_index = 0;
        }
        let mut samples = vec![0.0f32; length as usize];
        for current_sample in start_sample..end_sample {
            let current_time = current_sample as f64 / SAMPLE_RATE as f64;
            while handle.current_track_event_index < handle.current_track_events.len() {
                let (event_tick, event) =
                    &handle.current_track_events[handle.current_track_event_index];
                let event_time = handle.ticks_to_time(*event_tick);
                if event_time > current_time {
                    break; // No more events for this sample
                }

                // Process the event
                match event {
                    midi::NoteEvent::NoteOn(note) => {
                        handle.active_notes.insert(note.midi_note, note.clone());
                    }
                    midi::NoteEvent::NoteOff(midi_note) => {
                        handle.active_notes.remove(midi_note);
                    }
                }

                handle.current_track_event_index += 1;
            }
            let current_tempo = handle
                .tempo_index()
                .range(..OrderedFloat(current_time))
                .next_back()
                .expect("unreachable: tempo_index should not be empty")
                .1;
            // Generate audio for the active notes
            for (&midi_note, note) in &handle.active_notes {
                let frequency = midi::midi_note_to_freq(midi_note);
                let elapsed_ticks = current_time - note.start_tick as f64;
                let elapsed_seconds = elapsed_ticks / handle.ticks_per_beat as f64
                    * current_tempo.uspb as f64
                    / 1_000_000.0;
                let phase = (elapsed_seconds * frequency) as f32 * PI_2;
                let amplitude = note.velocity as f32 / 127.0; // Normalize velocity
                let sample_value = (phase.sin() * amplitude) as f32;
                samples[(current_sample - start_sample) as usize] += sample_value;
            }
        }
        handle.expected_next_sample = end_sample;

        Ok(samples.into_audio())
    }

    fn close(&self, _handle: Self::InputHandle) -> anyhow::Result<()> {
        Ok(())
    }

    fn set_audio_track(&self, handle: &mut Self::InputHandle, track: u32) -> anyhow::Result<u32> {
        let mid = handle.smf.borrow_mid();
        if track as usize >= mid.tracks.len() {
            return Err(anyhow::anyhow!("Track {} does not exist", track));
        }

        handle.current_track_events.clear();
        handle.current_track_event_index = 0;
        handle.active_notes.clear();
        let mut current_tick = 0u64;
        for event in mid.tracks[track as usize].iter() {
            current_tick += event.delta.as_int() as u64;
            if let midly::TrackEventKind::Midi { message, .. } = event.kind {
                if let midly::MidiMessage::NoteOn { key, vel: velocity } = message {
                    handle.current_track_events.push((
                        current_tick,
                        midi::NoteEvent::NoteOn(midi::Note {
                            midi_note: key.as_int(),
                            start_tick: current_tick,
                            velocity: velocity.as_int(),
                        }),
                    ));
                } else if let midly::MidiMessage::NoteOff { key, .. } = message {
                    handle
                        .current_track_events
                        .push((current_tick, midi::NoteEvent::NoteOff(key.as_int())));
                }
            }
        }

        Ok(track)
    }
}

aviutl2::register_input_plugin!(SinMidPlayerPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_midi() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("piano.mid");

        let plugin = SinMidPlayerPlugin::new();
        let handle = plugin.open(path).expect("Failed to open MIDI file");

        assert_eq!(handle.ticks_per_beat, 960);
    }
}

