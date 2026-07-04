use aviutl2::{
    AnyResult,
    filter::{FilterConfigItemSliceExt, FilterConfigItems},
};

#[aviutl2::filter::filter_config_items]
#[derive(Debug, Clone, PartialEq)]
pub struct MetronomeFilterConfig {
    #[track(name = "音量", range = 0.0..=1.0, step = 0.01, default = 0.8)]
    volume: f64,
    #[track(name = "周波数A(Hz)", range = 200.0..=2000.0, step = 1.0, default = 1000.0)]
    frequency_a: f64,
    #[track(name = "周波数B(Hz)", range = 200.0..=2000.0, step = 1.0, default = 800.0)]
    frequency_b: f64,
    #[track(name = "長さ(ms)", range = 5.0..=200.0, step = 1.0, default = 30.0)]
    click_ms: f64,
    #[file(name = "音源A", filters = { "WAVファイル" => ["wav"] })]
    sample_a: Option<std::path::PathBuf>,
    #[file(name = "音源B", filters = { "WAVファイル" => ["wav"] })]
    sample_b: Option<std::path::PathBuf>,
}

#[aviutl2::plugin(FilterPlugin)]
pub struct MetronomeFilter;

impl aviutl2::filter::FilterPlugin for MetronomeFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(Self)
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Rusty Metronome Filter".to_string(),
            label: None,
            information: format!(
                "Metronome effect, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/metronome-plugin",
                version = env!("CARGO_PKG_VERSION")
            ),
            flags: aviutl2::bitflag!(aviutl2::filter::FilterPluginFlags {
                audio: true,
                input: true,
            }),
            config_items: MetronomeFilterConfig::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &mut aviutl2::filter::FilterProcAudio,
    ) -> anyhow::Result<()> {
        let config: MetronomeFilterConfig = config.to_struct();
        let sample_rate = audio.scene.sample_rate;
        let sample_a = config
            .sample_a
            .as_deref()
            .and_then(|path| crate::wav::get_wav_sample(path, sample_rate));
        let sample_b = config
            .sample_b
            .as_deref()
            .and_then(|path| crate::wav::get_wav_sample(path, sample_rate));
        let bpm_info = crate::EDIT_HANDLE.call_read_section(|read| read.get_bpm_info())??;
        let mut bpm_grids = bpm_info
            .into_iter()
            .filter_map(BpmGrid::from_bpm_info)
            .collect::<Vec<_>>();
        if bpm_grids.is_empty() {
            return Ok(());
        }
        bpm_grids.sort_by(|a, b| a.offset.total_cmp(&b.offset));

        let info = crate::EDIT_HANDLE.get_edit_info();
        let object_start_time =
            audio.object.frame_s as f64 * *info.fps.denom() as f64 / *info.fps.numer() as f64;
        tracing::debug!(
            "frame_s: {}, fps: {}/{} => time_s: {}, bpm_grid_count: {}",
            audio.object.frame_s,
            info.fps.numer(),
            info.fps.denom(),
            object_start_time,
            bpm_grids.len(),
        );
        let click_length_samples = ((config.click_ms / 1000.0) * sample_rate as f64).round() as u64;
        if click_length_samples == 0 {
            return Ok(());
        }
        let mut lbuf = vec![0.0f32; audio.audio_object.sample_num as usize];
        let mut rbuf = vec![0.0f32; audio.audio_object.sample_num as usize];
        for i in 0..audio.audio_object.sample_num as usize {
            let current_sample_index = audio.audio_object.sample_index + i as u64;
            let current_time = object_start_time + current_sample_index as f64 / sample_rate as f64;
            let Some(bpm_grid) = get_bpm_grid_at(&bpm_grids, current_time) else {
                continue;
            };
            let Some((last_beat_sample_index, beat_number)) = get_last_beat_sample_index(
                sample_rate,
                bpm_grid.tempo,
                bpm_grid.offset - object_start_time,
                current_sample_index,
            ) else {
                continue;
            };
            let use_a = beat_number % bpm_grid.beat as i64 == 0;

            let sample_offset = current_sample_index.saturating_sub(last_beat_sample_index);
            let sample = if use_a {
                sample_a.as_deref()
            } else {
                sample_b.as_deref()
            };
            if let Some(sample) = sample {
                let sample_len = sample.len() as u64;
                if sample_offset < sample_len {
                    let index = sample_offset as usize;
                    let value_left = sample.left[index] * config.volume as f32;
                    let value_right = sample.right[index] * config.volume as f32;
                    lbuf[i] += clamp_sample(value_left);
                    rbuf[i] += clamp_sample(value_right);
                }
            } else if sample_offset < click_length_samples {
                let t = sample_offset as f32 / sample_rate as f32;
                let frequency = if use_a {
                    config.frequency_a as f32
                } else {
                    config.frequency_b as f32
                };
                let amplitude = (1.0 - (sample_offset as f32 / click_length_samples as f32))
                    * config.volume as f32;
                let value = (2.0 * std::f32::consts::PI * frequency * t).sin() * amplitude;
                lbuf[i] += clamp_sample(value * 0.5);
                rbuf[i] += clamp_sample(value * 0.5);
            }
        }

        audio.set_sample_data(aviutl2::filter::AudioChannel::Left, &lbuf);
        audio.set_sample_data(aviutl2::filter::AudioChannel::Right, &rbuf);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct BpmGrid {
    tempo: f64,
    beat: i32,
    offset: f64,
}

impl BpmGrid {
    fn from_bpm_info(info: aviutl2::generic::BpmInfo) -> Option<Self> {
        if info.tempo < 0.1 || info.beat <= 0 {
            return None;
        }
        Some(Self {
            tempo: info.tempo as f64,
            beat: info.beat,
            offset: info.offset,
        })
    }
}

fn get_bpm_grid_at(bpm_grids: &[BpmGrid], time: f64) -> Option<BpmGrid> {
    let index = bpm_grids.partition_point(|grid| grid.offset <= time);
    if index == 0 {
        None
    } else {
        Some(bpm_grids[index - 1])
    }
}

fn get_last_beat_sample_index(
    sample_rate: u32,
    bpm: f64,
    bpm_offset: f64,
    current_sample_index: u64,
) -> Option<(u64, i64)> {
    let samples_per_beat = (60.0 / bpm) * sample_rate as f64;
    let offset_samples = bpm_offset * sample_rate as f64;
    let adjusted_index = current_sample_index as f64 - offset_samples;
    let beat_count = (adjusted_index / samples_per_beat).floor();
    let last_beat_sample_index = (beat_count * samples_per_beat + offset_samples).round();
    if last_beat_sample_index < 0.0 || last_beat_sample_index > current_sample_index as f64 {
        return None;
    }
    Some((last_beat_sample_index as u64, beat_count as i64))
}

fn clamp_sample(value: f32) -> f32 {
    value.clamp(-1.0, 1.0)
}
