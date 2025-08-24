use itertools::Itertools;

use ordered_float::OrderedFloat;

pub const US_PER_SECOND: u64 = 1_000_000;

#[allow(dead_code)]
mod owned_smf {
    use anyhow::Context;

    #[ouroboros::self_referencing(pub_extras)]
    #[derive(Debug)]
    pub struct OwnedSmf {
        pub content: Vec<u8>,
        #[borrows(content)]
        #[covariant]
        pub mid: midly::Smf<'this>,
    }

    impl OwnedSmf {
        pub fn from_content(content: Vec<u8>) -> anyhow::Result<Self> {
            let smf = OwnedSmfTryBuilder {
                content,
                mid_builder: |content| midly::Smf::parse(content),
            }
            .try_build()
            .context("Failed to parse MIDI data")?;
            Ok(smf)
        }
    }
}

pub use owned_smf::OwnedSmf;

#[derive(Debug, Clone)]
pub struct Note {
    pub midi_note: u8,
    pub velocity: u8, // MIDI velocity for amplitude
}

#[derive(Debug, Clone)]
pub enum NoteEvent {
    NoteOn(Note),
    NoteOff(u8),
}

#[derive(Debug, Clone)]
pub struct TempoIndexCacheEntry {
    ticks: u64,
    uspb: u64,
}

#[derive(Debug, Clone)]
pub struct TempoIndex {
    pub index: std::collections::BTreeMap<OrderedFloat<f64>, TempoIndexCacheEntry>,

    ticks_per_beat: u64,
}

impl TempoIndex {
    pub fn new(mid: &OwnedSmf, ticks_per_beat: u64) -> Self {
        let mid = mid.borrow_mid();
        let tempo_changes = {
            let mut tempo_changes = std::collections::BTreeMap::new();
            for track in &mid.tracks {
                let mut current_tick = 0u64;
                for event in track.iter() {
                    current_tick += event.delta.as_int() as u64;
                    if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(uspb)) =
                        &event.kind
                    {
                        tempo_changes.insert(current_tick, uspb.as_int() as u64);
                    }
                }
            }
            tempo_changes
                .entry(0)
                .or_insert_with(|| US_PER_SECOND * 60 / 120);

            tempo_changes.into_iter().collect::<Vec<_>>()
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
                ((*p_uspb as f64) / (US_PER_SECOND as f64)) * (delta_ticks / ticks_per_beat) as f64;
            current_time += delta_time;
            tempo_index.insert(
                OrderedFloat(current_time),
                TempoIndexCacheEntry {
                    ticks: *ticks,
                    uspb: *uspb,
                },
            );
        }

        TempoIndex {
            index: tempo_index,
            ticks_per_beat,
        }
    }

    pub fn ticks_to_time(&self, ticks: u64) -> f64 {
        if ticks == 0 {
            return 0.0; // 0 ticks corresponds to 0 seconds
        }
        let (last_time, last_entry) = self.index.iter().rfind(|(_, entry)| entry.ticks <= ticks).expect(
            "unreachable: there should always be an entry with ticks <= the given ticks (at least the first entry with 0 ticks)",
        );
        let delta_ticks = ticks - last_entry.ticks;
        let delta_time = (last_entry.uspb as f64 / US_PER_SECOND as f64)
            * (delta_ticks as f64 / self.ticks_per_beat as f64);

        last_time.into_inner() + delta_time
    }
}

#[derive(Debug, Clone)]
pub struct Track {
    pub events: Vec<(f64, crate::track::NoteEvent)>,
}

impl Track {
    pub fn new(smf: &OwnedSmf, track: u32, tempo_index: &TempoIndex) -> anyhow::Result<Self> {
        let mid = smf.borrow_mid();
        if track >= mid.tracks.len() as u32 {
            return Err(anyhow::anyhow!("Track {} does not exist", track));
        }

        let mut key_state = std::collections::HashMap::new();
        let mut current_track_events = vec![];

        let mut events = vec![];
            let mut current_tick = 0u64;
            for event in mid.tracks[track as usize].iter() {
                current_tick += event.delta.as_int() as u64;
                events.push((current_tick, event));
            }
        for (current_tick, event) in events {
            if let midly::TrackEventKind::Midi { message, .. } = event.kind {
                if let midly::MidiMessage::NoteOn { key, vel: velocity } = message {
                    key_state
                        .entry(key.as_int())
                        .and_modify(|v| *v += 1)
                        .or_insert(1);
                    if key_state[&key.as_int()] == 1 {
                        let event_time = tempo_index.ticks_to_time(current_tick);
                        current_track_events.push((
                            event_time,
                            NoteEvent::NoteOn(Note {
                                midi_note: key.as_int(),
                                velocity: velocity.as_int(),
                            }),
                        ));
                    }
                } else if let midly::MidiMessage::NoteOff { key, .. } = message {
                    if *key_state
                        .entry(key.as_int())
                        .and_modify(|v| *v -= 1)
                        .or_insert(0)
                        == 0
                    {
                        let event_time = tempo_index.ticks_to_time(current_tick);
                        current_track_events.push((event_time, NoteEvent::NoteOff(key.as_int())));
                    }
                }
            }
        }

        Ok(Track {
            events: current_track_events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_midi() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join("library_w4nderers.mid");

        let content = std::fs::read(path).unwrap();
        let smf = OwnedSmf::from_content(content).unwrap();

        let ticks_per_beat = match smf.borrow_mid().header.timing {
            midly::Timing::Metrical(tpb) => tpb.as_int() as u64,
            _ => {
                panic!("Only Metrical timing is supported");
            }
        };
        let tempo_index = TempoIndex::new(&smf, ticks_per_beat);
        let track = Track::new(&smf, 1, &tempo_index).unwrap();
        assert!(!track.events.is_empty());
    }
}
