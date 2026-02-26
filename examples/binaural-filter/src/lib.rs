use aviutl2::{
    filter::{FilterConfigItemSliceExt, FilterConfigItems},
    tracing,
};
use ringbuffer::RingBuffer;

#[aviutl2::filter::filter_config_items]
#[derive(Debug, Clone, PartialEq)]
pub struct FilterConfig {
    #[track(name = "ゲイン", range = 0.0..=8.0, step = 0.01, default = 2.0)]
    gain: f32,
    #[track(name = "横回転", range = -180.0..=180.0, step = 1.0, default = 0.0)]
    rotate_yaw: f64,
    #[track(name = "縦回転", range = -90.0..=90.0, step = 1.0, default = 0.0)]
    rotate_pitch: f64,
}

static HRIR_SPHERE: std::sync::LazyLock<hrtf::HrirSphere> = std::sync::LazyLock::new(|| {
    let reader = std::io::Cursor::new(include_bytes!(env!("HRIR_PATH")));
    hrtf::HrirSphere::new(reader, 44100).expect("Failed to load HRIR data")
});

fn resample_size(frames: usize, input_rate: usize, output_rate: usize) -> usize {
    if input_rate == output_rate {
        return frames;
    }
    let gcd = num_integer::gcd(input_rate, output_rate);
    let up = output_rate / gcd;
    let down = input_rate / gcd;
    (frames * up).div_ceil(down)
}
fn linear_resample(input: &[f32], output: &mut [f32]) {
    if input.len() == output.len() {
        output.copy_from_slice(input);
        return;
    }
    let input_len = input.len() as f64;
    let output_len = output.len() as f64;
    for (i, out_sample) in output.iter_mut().enumerate() {
        let pos = (i as f64) * input_len / output_len;
        let idx = pos.floor() as usize;
        let frac = pos - (idx as f64);
        if idx + 1 < input.len() {
            *out_sample = input[idx] * (1.0 - frac) as f32 + input[idx + 1] * (frac) as f32;
        } else if idx < input.len() {
            *out_sample = input[idx];
        } else {
            *out_sample = 0.0;
        }
    }
}
// fn resample(input: &[f32], input_rate: usize, output_rate: usize) -> Vec<f32> {
//     if input_rate == output_rate {
//         return input.to_vec();
//     }
//     let gcd = num_integer::gcd(input_rate, output_rate);
//     let up = output_rate / gcd;
//     let down = input_rate / gcd;
//     let mut output = vec![0.0; (input.len() * up).div_ceil(down)];
//     for (i, &sample) in input.iter().enumerate() {
//         output[i * up / down] += sample;
//     }
//     output
// }

struct BinauralStates {
    hrtf: hrtf::HrtfProcessor,
    num_blocks: usize,
    block_size: usize,
    audio_cache: ringbuffer::AllocRingBuffer<f32>,
    requested_sample_count: usize,
    tail_index: usize,

    prev_left_samples: Vec<f32>,
    prev_right_samples: Vec<f32>,
}
impl BinauralStates {
    fn new(frame_size: usize, sample_rate: f64) -> anyhow::Result<Self> {
        let frame_44100_size = resample_size(frame_size, sample_rate as usize, 44100);
        let mut num_blocks = 2_usize.pow(3);
        let mut block_size = next_pow2(frame_44100_size) / (num_blocks / 2);
        if block_size == 0 {
            block_size = HRIR_SPHERE.len();
        }
        while block_size < HRIR_SPHERE.len() {
            block_size *= 2;
            num_blocks /= 2;
        }
        if num_blocks < 4 {
            num_blocks = 4;
        }
        let hrtf = hrtf::HrtfProcessor::new(HRIR_SPHERE.clone(), num_blocks, block_size);

        let cache_size = num_blocks * block_size * 16;

        let mut audio_cache = ringbuffer::AllocRingBuffer::new(cache_size);
        audio_cache.extend((0..cache_size).map(|_| 0.0));
        tracing::debug!(
            "BinauralStates::new: frame_size={}, frame_44100_size={}, block_size={}, cache_size={}",
            frame_size,
            frame_44100_size,
            block_size,
            cache_size
        );

        Ok(Self {
            hrtf,
            num_blocks,
            block_size,
            audio_cache,
            requested_sample_count: resample_size(
                num_blocks * block_size,
                44100,
                sample_rate as usize,
            ),
            tail_index: 0,
            prev_left_samples: vec![],
            prev_right_samples: vec![],
        })
    }

    fn process(
        &mut self,
        audio: &[f32],
        gain: f32,
        rotate_yaw: f64,
        rotate_pitch: f64,
    ) -> anyhow::Result<(Vec<f32>, Vec<f32>)> {
        assert_eq!(audio.len(), self.requested_sample_count);
        // NOTE: 17.0はおまじない
        let radians_yaw = (rotate_yaw + 17.0).to_radians();
        let radians_pitch = rotate_pitch.to_radians();
        let (x, y, z) = (
            (1.0 * radians_pitch.cos() * radians_yaw.sin()) as f32,
            (1.0 * radians_pitch.sin()) as f32,
            (1.0 * radians_pitch.cos() * radians_yaw.cos()) as f32,
        );
        let mut source = vec![0.0f32; self.block_size * self.num_blocks];
        linear_resample(audio, &mut source[..]);
        let mut output = vec![(0.0, 0.0); self.block_size * self.num_blocks];

        let context = hrtf::HrtfContext {
            source: &source,
            output: &mut output,
            new_sample_vector: hrtf::Vec3 { x, y, z },
            prev_sample_vector: hrtf::Vec3 { x, y, z },
            prev_left_samples: &mut self.prev_left_samples,
            prev_right_samples: &mut self.prev_right_samples,
            new_distance_gain: gain,
            prev_distance_gain: gain,
        };

        self.hrtf.process_samples(context);

        let (left_44100, right_44100): (Vec<f32>, Vec<f32>) = output.into_iter().unzip();
        let mut left = vec![0.0; audio.len()];
        let mut right = vec![0.0; audio.len()];
        linear_resample(&left_44100, &mut left);
        linear_resample(&right_44100, &mut right);
        Ok((left, right))
    }
}

#[aviutl2::plugin(FilterPlugin)]
struct BinauralFilter {
    states: dashmap::DashMap<i64, BinauralStates>,
}

impl aviutl2::filter::FilterPlugin for BinauralFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        aviutl2::tracing_subscriber::fmt()
            .with_max_level(if cfg!(debug_assertions) {
                tracing::Level::DEBUG
            } else {
                tracing::Level::INFO
            })
            .event_format(aviutl2::logger::AviUtl2Formatter)
            .with_writer(aviutl2::logger::AviUtl2LogWriter)
            .init();
        Ok(Self {
            states: dashmap::DashMap::new(),
        })
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Rusty Binaural Filter".to_string(),
            label: None,
            information: format!(
                "Binaural filter, powered by hrtf crate, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/equalizer-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            flags: aviutl2::bitflag!(aviutl2::filter::FilterPluginFlags { audio: true }),
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &mut aviutl2::filter::FilterProcAudio,
    ) -> anyhow::Result<()> {
        let config: FilterConfig = config.to_struct();
        let obj_id = audio.object.effect_id;

        let num_samples = audio.audio_object.sample_num as usize;
        if num_samples == 0 {
            tracing::warn!("num_samples is zero");
            return Ok(());
        }
        let mut states = self.states.entry(obj_id).or_try_insert_with(|| {
            BinauralStates::new(num_samples, audio.scene.sample_rate as f64)
        })?;
        if (((states.requested_sample_count as f32) * (3.0 / 4.0)) as usize) < num_samples {
            tracing::info!(
                "Frame size changed: {} -> {}",
                states.requested_sample_count,
                num_samples
            );
            *states = BinauralStates::new(num_samples, audio.scene.sample_rate as f64)?;
        }
        let mut left_samples = vec![0.0f32; num_samples];
        let mut right_samples = vec![0.0f32; num_samples];
        audio.get_sample_data(aviutl2::filter::AudioChannel::Left, &mut left_samples);
        audio.get_sample_data(aviutl2::filter::AudioChannel::Right, &mut right_samples);

        let cache_start = (states.tail_index as i64) - (states.audio_cache.len() as i64);
        let expected_start = (audio.audio_object.sample_index as i64) + (num_samples as i64)
            - (states.requested_sample_count as i64);

        if (audio.audio_object.sample_index as i64) <= cache_start
            || (states.tail_index as i64) < expected_start
            || (states.tail_index < audio.audio_object.sample_index as usize)
            || expected_start < cache_start
        {
            tracing::info!(
                "Cache reset: sample_index={}, tail_index={}, cache_start={}, expected_start={}",
                audio.audio_object.sample_index,
                states.tail_index,
                cache_start,
                expected_start,
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

        let cache_start = (states.tail_index as i64) - (states.audio_cache.len() as i64);
        let expected_start = (audio.audio_object.sample_index as i64) + (num_samples as i64)
            - (states.requested_sample_count as i64);
        let samples = states
            .audio_cache
            .iter()
            .skip((expected_start - cache_start) as usize)
            .take(states.requested_sample_count)
            .copied()
            .collect::<Vec<_>>();

        let (new_left, new_right) = states.process(
            &samples,
            config.gain,
            config.rotate_yaw,
            config.rotate_pitch,
        )?;
        let new_left = &new_left[(new_left.len() - num_samples)..];
        let new_right = &new_right[(new_right.len() - num_samples)..];
        audio.set_sample_data(aviutl2::filter::AudioChannel::Left, new_left);
        audio.set_sample_data(aviutl2::filter::AudioChannel::Right, new_right);

        Ok(())
    }
}

fn next_pow2(n: usize) -> usize {
    let mut m = 1;
    while m < n {
        m *= 2;
    }
    m
}

aviutl2::register_filter_plugin!(BinauralFilter);
