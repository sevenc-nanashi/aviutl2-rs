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
                as_object: true,
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
        let info = crate::EDIT_HANDLE.get().unwrap().get_edit_info();
        let bpm = info.grid_bpm_tempo;
        if bpm < 0.1 {
            return Ok(());
        }
        let bpm_offset = info.grid_bpm_offset;
        let beat_count = info.grid_bpm_beat;
        if beat_count == 0 {
            return Ok(());
        }
        let click_length_samples = ((config.click_ms / 1000.0) * sample_rate as f64).round() as u64;
        if click_length_samples == 0 {
            return Ok(());
        }
        let mut lbuf = vec![0.0f32; audio.audio_object.sample_num as usize];
        let mut rbuf = vec![0.0f32; audio.audio_object.sample_num as usize];
        for i in 0..audio.audio_object.sample_num as usize {
            let current_sample_index = audio.audio_object.sample_index + i as u64;
            let (last_beat_sample_index, beat_number) =
                get_last_beat_sample_index(sample_rate, bpm, bpm_offset, current_sample_index);
            let use_a = beat_number % beat_count as i64 == 0;

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

fn get_last_beat_sample_index(
    sample_rate: u32,
    bpm: f32,
    bpm_offset: f32,
    current_sample_index: u64,
) -> (u64, i64) {
    let samples_per_beat = (60.0 / bpm) * sample_rate as f32;
    let offset_samples = bpm_offset * sample_rate as f32;
    let adjusted_index = current_sample_index as f32 - offset_samples;
    let beat_count = (adjusted_index / samples_per_beat).floor();
    let last_beat_sample_index = (beat_count * samples_per_beat + offset_samples).round() as u64;
    (last_beat_sample_index, beat_count as i64)
}

fn clamp_sample(value: f32) -> f32 {
    value.clamp(-1.0, 1.0)
}
