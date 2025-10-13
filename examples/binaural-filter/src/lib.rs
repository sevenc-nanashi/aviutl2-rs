use aviutl2::{
    filter::{FilterConfigItemSliceExt, FilterConfigItems},
    log,
};
use itertools::Itertools;
use ringbuffer::RingBuffer;

#[derive(Debug, Clone, PartialEq, aviutl2::filter::FilterConfigItems)]
pub struct FilterConfig {
    #[track(name = "距離", range = 0.0..=1.0, step = 0.01, default = 1.0)]
    distance: f64,
    #[track(name = "Yaw", range = -180.0..=180.0, step = 1.0, default = 0.0)]
    rotate_yaw: f64,
    #[track(name = "Pitch", range = -90.0..=90.0, step = 1.0, default = 0.0)]
    rotate_pitch: f64,
}

struct BinauralStates {
    hrtf: audionimbus::Hrtf,
    effect: audionimbus::BinauralEffect,
    frame_size: usize,
    audio_cache: ringbuffer::AllocRingBuffer<f32>,
    tail_index: usize,
}
impl BinauralStates {
    fn new(
        context: &audionimbus::Context,
        sample_rate: f64,
        frame_size: usize,
        cache_size: usize,
    ) -> anyhow::Result<Self> {
        let audio_settings = audionimbus::AudioSettings {
            sampling_rate: sample_rate as usize,
            frame_size,
        };
        let hrtf = audionimbus::Hrtf::try_new(
            context,
            &audio_settings,
            &audionimbus::HrtfSettings::default(),
        )?;
        let binaural_effect = audionimbus::BinauralEffect::try_new(
            context,
            &audio_settings,
            &audionimbus::BinauralEffectSettings { hrtf: &hrtf },
        )?;

        let mut audio_cache = ringbuffer::AllocRingBuffer::new(cache_size);
        audio_cache.extend((0..cache_size).map(|_| 0.0));

        Ok(Self {
            hrtf,
            effect: binaural_effect,
            frame_size,
            audio_cache,
            tail_index: 0,
        })
    }

    fn process(
        &mut self,
        audio: &[f32],
        distance: f64,
        rotate_yaw: f64,
        rotate_pitch: f64,
    ) -> anyhow::Result<(Vec<f32>, Vec<f32>)> {
        anyhow::ensure!(audio.len() == self.frame_size);
        let input_buffer = audionimbus::AudioBuffer::try_with_data(audio)?;
        let mut output = vec![0.0; audio.len() * 2];
        let output_buffer = audionimbus::AudioBuffer::try_with_data_and_settings(
            &mut output,
            &audionimbus::AudioBufferSettings {
                num_channels: Some(2),
                ..Default::default()
            },
        )?;

        let radians_yaw = rotate_yaw.to_radians();
        let radians_pitch = rotate_pitch.to_radians();
        let (x, y, z) = (
            distance * radians_pitch.cos() * radians_yaw.sin(),
            distance * radians_pitch.sin(),
            distance * radians_pitch.cos() * radians_yaw.cos(),
        );
        let direction = audionimbus::Direction::new(x as f32, y as f32, z as f32);
        let binaural_effect_params = audionimbus::BinauralEffectParams {
            direction,
            interpolation: audionimbus::HrtfInterpolation::Bilinear,
            spatial_blend: 1.0,
            hrtf: &self.hrtf,
            peak_delays: None,
        };
        self.effect
            .apply(&binaural_effect_params, &input_buffer, &output_buffer);

        Ok(output_buffer
            .channels()
            .map(|ch| ch.to_vec())
            .collect_tuple()
            .unwrap())
    }
}

struct BinauralFilter {
    context: audionimbus::Context,
    states: dashmap::DashMap<i64, BinauralStates>,
}

impl aviutl2::filter::FilterPlugin for BinauralFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        env_logger::Builder::new()
            .parse_filters("info")
            .target(aviutl2::utils::debug_logger_target())
            .init();
        Ok(Self {
            context: audionimbus::Context::try_new(&audionimbus::ContextSettings::default())?,
            states: dashmap::DashMap::new(),
        })
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Rusty Binaural Filter".to_string(),
            label: None,
            information: format!(
                "Binaural filter, powered by Steam Audio, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/equalizer-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            filter_type: aviutl2::filter::FilterType::Audio,
            as_object: false,
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &aviutl2::filter::FilterProcAudio,
    ) -> anyhow::Result<()> {
        let config: FilterConfig = config.to_struct();
        let obj_id = audio.object.id;

        let num_samples = audio.audio_object.sample_num as usize;
        let sample_rate = audio.scene.sample_rate as f64;
        let mut states = self.states.entry(obj_id).or_try_insert_with(|| {
            BinauralStates::new(
                &self.context,
                sample_rate,
                larger_min_pow2(num_samples) * 2,
                audio.scene.sample_rate as usize * 10,
            )
        })?;
        if states.frame_size < num_samples {
            log::info!(
                "Frame size changed: {} -> {}",
                states.frame_size,
                num_samples
            );
            *states = BinauralStates::new(
                &self.context,
                sample_rate,
                larger_min_pow2(num_samples) * 2,
                audio.scene.sample_rate as usize * 10,
            )?;
        }
        let left_samples = audio.get_sample_data(aviutl2::filter::AudioChannel::Left);
        let right_samples = audio.get_sample_data(aviutl2::filter::AudioChannel::Right);

        if (audio.audio_object.sample_index as i64)
            <= (states.tail_index as i64) - (states.audio_cache.len() as i64)
            || (states.tail_index as i64)
                < (audio.audio_object.sample_index as i64 + num_samples as i64
                    - states.frame_size as i64)
        {
            log::info!(
                "Cache reset: sample_index={}, tail_index={}, cache_length={}",
                audio.audio_object.sample_index,
                states.tail_index,
                states.audio_cache.len()
            );
            let cache_length = states.audio_cache.len();
            states.tail_index = audio.audio_object.sample_index as usize;
            states.audio_cache.clear();
            states.audio_cache.extend((0..cache_length).map(|_| 0.0));
        }

        let mono_samples: Vec<f32> = left_samples
            .iter()
            .zip(right_samples.iter())
            .map(|(l, r)| 0.5 * (l + r))
            .collect();
        let last_index = (audio.audio_object.sample_index as usize) + num_samples;
        let uncached_samples = last_index.saturating_sub(states.tail_index);
        if uncached_samples > 0 {
            states.audio_cache.extend(
                mono_samples
                    .iter()
                    .skip(num_samples - uncached_samples)
                    .take(uncached_samples)
                    .copied(),
            );
            states.tail_index += uncached_samples;
        }

        let frame_start =
            audio.audio_object.sample_index as i64 + num_samples as i64 - states.frame_size as i64;
        let samples = states
            .audio_cache
            .iter()
            .skip(
                (frame_start - (states.tail_index as i64 - states.audio_cache.len() as i64)).max(0)
                    as usize,
            )
            .take(states.frame_size)
            .copied()
            .collect::<Vec<_>>();
        let (new_left, new_right) = states.process(
            &samples,
            config.distance,
            config.rotate_yaw,
            config.rotate_pitch,
        )?;
        let new_left = &new_left[(new_left.len() - num_samples)..];
        let new_right = &new_right[(new_right.len() - num_samples)..];
        assert!(new_left.len() == num_samples);
        assert!(new_right.len() == num_samples);
        audio.set_sample_data(new_left, aviutl2::filter::AudioChannel::Left);
        audio.set_sample_data(new_right, aviutl2::filter::AudioChannel::Right);

        Ok(())
    }
}

fn larger_min_pow2(n: usize) -> usize {
    let mut m = 1;
    while m < n {
        m *= 2;
    }
    m
}

aviutl2::register_filter_plugin!(BinauralFilter);
