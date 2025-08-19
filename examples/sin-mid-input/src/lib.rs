use aviutl2::input::InputPlugin;
use aviutl2::input::IntoAudio;
use itertools::Itertools;
use ordered_float::OrderedFloat;

struct SinMidPlayerPlugin {}

#[ouroboros::self_referencing]
struct SinMidPlayerHandle {
    content: Vec<u8>,
    tempo_index_cache:
        std::sync::OnceLock<std::collections::BTreeMap<OrderedFloat<f64>, TempoIndexCacheEntry>>,

    #[borrows(content)]
    #[covariant]
    mid: midly::Smf<'this>,
}
#[derive(Debug, Clone)]
struct TempoIndexCacheEntry {
    ticks: u64,
    uspb: u64,
}

impl SinMidPlayerHandle {
    fn tempo_index(&self) -> &std::collections::BTreeMap<OrderedFloat<f64>, TempoIndexCacheEntry> {
        self.borrow_tempo_index_cache().get_or_init(|| {
            let mid = self.borrow_mid();
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

            let midly::Timing::Metrical(ticks_per_beat) = mid.header.timing else {
                unreachable!("unreachable: should be validated before this point");
            };
            let ticks_per_beat = ticks_per_beat.as_int() as u64;
            let mut current_time = 0f64;
            for ((p_ticks, p_uspb), (ticks, uspb)) in tempo_changes.iter().tuple_windows() {
                let delta_ticks = ticks - p_ticks;
                let delta_time =
                    ((*p_uspb as f64) / 1_000_000.0) * (delta_ticks / ticks_per_beat) as f64;
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
}

impl InputPlugin for SinMidPlayerPlugin {
    type InputHandle = SinMidPlayerHandle;

    fn new() -> Self {
        SinMidPlayerPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::input::InputPluginTable {
        aviutl2::input::InputPluginTable {
            name: "SinMid Player Plugin".to_string(),
            input_type: aviutl2::input::InputType::Video,
            file_filters: vec![aviutl2::FileFilter {
                name: "MIDI Files".to_string(),
                extensions: vec!["mid".to_string()],
            }],
            information: format!(
                "SinMid Player for AviUtl / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/sin-mid-input",
                version = env!("CARGO_PKG_VERSION")
            ),
            concurrent: false,
            can_config: false,
        }
    }

    fn open(&self, file: std::path::PathBuf) -> aviutl2::AnyResult<Self::InputHandle> {
        let content =
            std::fs::read(file).map_err(|e| anyhow::anyhow!("Failed to read MIDI file: {}", e))?;
        let handle = SinMidPlayerHandleTryBuilder {
            content,
            tempo_index_cache: std::sync::OnceLock::new(),
            mid_builder: |content| midly::Smf::parse(content),
        }
        .try_build()?;
        {
            let mid = handle.borrow_mid();
            if !matches!(mid.header.timing, midly::Timing::Metrical(_)) {
                return Err(anyhow::anyhow!("Only Metrical timing is supported"));
            }
        }

        aviutl2::odbg!(handle.tempo_index());

        Ok(handle)
    }

    fn get_input_info(
        &self,
        handle: &Self::InputHandle,
    ) -> aviutl2::AnyResult<aviutl2::input::InputInfo> {
        let mid = handle.borrow_mid();
        let num_tracks = mid.tracks.len() as u32;
        let num_frames = mid
            .tracks
            .iter()
            .map(|t| t.iter().map(|e| e.delta.as_int() as u64).sum::<u64>())
            .max()
            .unwrap_or(0);

        Ok(aviutl2::input::InputInfo {
            video: None,
            audio: None, // No audio for MIDI files
        })
    }

    fn read_audio_mut(
        &self,
        handle: &mut Self::InputHandle,
        start: i32,
        length: i32,
        track: u32,
    ) -> anyhow::Result<impl aviutl2::input::IntoAudio> {
        Ok(vec![0u8; 0].into_audio())
    }

    fn close(&self, handle: Self::InputHandle) -> anyhow::Result<()> {
        Ok(())
    }
}

aviutl2::register_input_plugin!(SinMidPlayerPlugin);
