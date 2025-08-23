mod synthesizer;
mod track;

use aviutl2::input::{InputPlugin, IntoAudio};
use synthesizer::SAMPLE_RATE;

struct MidiPlayerPlugin {}
const TAIL_LENGTH: f64 = 1.0; // 1 second tail length

#[derive(Debug)]
struct MidiPlayerHandle {
    smf: track::OwnedSmf,
    track_number: u32,
    tempo_index: track::TempoIndex,
    // プレビュー生成と波形生成のそれぞれに1つずつ使用する
    synthesizers: [synthesizer::Synthesizer; 2],
}
impl MidiPlayerHandle {
    fn open(content: Vec<u8>) -> anyhow::Result<Self> {
        let smf = track::OwnedSmf::from_content(content)?;

        let ticks_per_beat = match smf.borrow_mid().header.timing {
            midly::Timing::Metrical(tpb) => tpb.as_int() as u64,
            timing => {
                return Err(anyhow::anyhow!("Only Metrical timing is supported, got {timing:?}"));
            }
        };

        let tempo_index = track::TempoIndex::new(&smf, ticks_per_beat);
        let track = track::Track::new(&smf, 0, &tempo_index)?; // Default to the first track
        let track = std::sync::Arc::new(track);

        Ok(MidiPlayerHandle {
            smf,
            track_number: 0, // Default to the first track
            tempo_index,
            synthesizers: [
                synthesizer::Synthesizer::new(std::sync::Arc::clone(&track))?,
                synthesizer::Synthesizer::new(std::sync::Arc::clone(&track))?,
            ],
        })
    }
}

impl InputPlugin for MidiPlayerPlugin {
    type InputHandle = MidiPlayerHandle;

    fn new() -> Self {
        MidiPlayerPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "Midi Piano Player Plugin".to_string(),
            input_type: aviutl2::input::InputType::Audio,
            file_filters: vec![aviutl2::FileFilter {
                name: "MIDI Files".to_string(),
                extensions: vec!["mid".to_string()],
            }],
            information: format!(
                "Midi Piano Player for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/midi-player-input",
                version = env!("CARGO_PKG_VERSION")
            ),
            concurrent: false,
            // TODO: sf2の設定を可能にする
            can_config: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> aviutl2::AnyResult<Self::InputHandle> {
        let content =
            std::fs::read(file).map_err(|e| anyhow::anyhow!("Failed to read MIDI file: {}", e))?;
        let handle = MidiPlayerHandle::open(content)
            .map_err(|e| anyhow::anyhow!("Failed to open MIDI file: {}", e))?;

        aviutl2::odbg!(&handle.tempo_index);

        Ok(handle)
    }

    fn get_track_count(&self, handle: &mut Self::InputHandle) -> anyhow::Result<(u32, u32)> {
        let mid = handle.smf.borrow_mid();
        let video_tracks = 0; // MIDI does not have video tracks
        let audio_tracks = mid.tracks.len() as u32;

        Ok((video_tracks, audio_tracks + 1))
    }

    fn get_input_info(
        &self,
        handle: &mut Self::InputHandle,
        _video_track: u32,
        audio_track: u32,
    ) -> anyhow::Result<aviutl2::input::InputInfo> {
        let mid = handle.smf.borrow_mid();
        let last_ticks = mid
            .tracks
            .iter()
            .map(|t| t.iter().map(|e| e.delta.as_int() as u64).sum::<u64>())
            .max()
            .unwrap_or(0);

        handle.track_number = audio_track;
        let track = track::Track::new(&handle.smf, audio_track, &handle.tempo_index)?;
        let track = std::sync::Arc::new(track);
        for synth in &mut handle.synthesizers {
            synth.set_track(std::sync::Arc::clone(&track));
        }

        Ok(aviutl2::input::InputInfo {
            video: None,
            audio: Some(aviutl2::input::AudioInputInfo {
                sample_rate: SAMPLE_RATE,
                channels: 2, // Mono output
                num_samples: ((handle.tempo_index.ticks_to_time(last_ticks) + TAIL_LENGTH).max(0.0)
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
    ) -> anyhow::Result<impl aviutl2::input::IntoAudio> {
        let synth = &mut handle
            .synthesizers
            .iter_mut()
            .min_by_key(|s| (s.expected_next_sample as i64 - start as i64).abs())
            .unwrap();
        let start_sample = start as u64;
        let end_sample = start_sample + length as u64;
        let samples_between = start_sample as i64 - synth.expected_next_sample as i64;
        if samples_between < -(SAMPLE_RATE as f64 * 0.01) as i64
            || samples_between > (SAMPLE_RATE as f64 * 0.01) as i64
        {
            // 再生位置が飛んだのでリセット
            synth.reset();
        }
        let samples = synth.render(length, start_sample, end_sample);

        Ok(samples.into_audio())
    }

    fn close(&self, _handle: Self::InputHandle) -> anyhow::Result<()> {
        Ok(())
    }
}

aviutl2::register_input_plugin!(MidiPlayerPlugin);
