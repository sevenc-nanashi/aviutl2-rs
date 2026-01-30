use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

#[derive(Debug, Clone)]
pub(crate) struct SampleData {
    pub(crate) left: Vec<f32>,
    pub(crate) right: Vec<f32>,
}

impl SampleData {
    pub(crate) fn len(&self) -> usize {
        self.left.len().min(self.right.len())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct SampleCacheKey {
    path: PathBuf,
    sample_rate: u32,
}

static SAMPLE_CACHE: LazyLock<dashmap::DashMap<SampleCacheKey, Option<Arc<SampleData>>>> =
    LazyLock::new(dashmap::DashMap::new);

pub(crate) fn get_wav_sample(path: &Path, target_rate: u32) -> Option<Arc<SampleData>> {
    let key = SampleCacheKey {
        path: path.to_path_buf(),
        sample_rate: target_rate,
    };
    let entry = SAMPLE_CACHE.entry(key).or_insert_with(|| {
        let result = load_wav_sample(path, target_rate);
        match result {
            Ok(sample) => Some(Arc::new(sample)),
            Err(err) => {
                log::error!("Failed to load WAV sample from {:?}: {}", path, err);
                None
            }
        }
    });
    entry.value().as_ref().cloned()
}

fn load_wav_sample(path: &Path, target_rate: u32) -> anyhow::Result<SampleData> {
    let (left, right, input_rate) = load_wav_channels(path)?;
    let left = resample_channel(&left, input_rate, target_rate);
    let right = resample_channel(&right, input_rate, target_rate);
    Ok(SampleData { left, right })
}

fn load_wav_channels(path: &Path) -> anyhow::Result<(Vec<f32>, Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    if channels == 0 || channels > 2 {
        anyhow::bail!("Unsupported channel count: {}", channels);
    }
    let mut left = Vec::new();
    let mut right = Vec::new();
    match spec.sample_format {
        hound::SampleFormat::Float => {
            for (index, sample) in reader.samples::<f32>().enumerate() {
                let value = sample?;
                push_sample(&mut left, &mut right, channels, index, value);
            }
        }
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            if bits <= 16 {
                let max = i16::MAX as f32;
                for (index, sample) in reader.samples::<i16>().enumerate() {
                    let value = sample? as f32 / max;
                    push_sample(&mut left, &mut right, channels, index, value);
                }
            } else {
                let max = ((1u64 << (bits - 1)) - 1) as f32;
                for (index, sample) in reader.samples::<i32>().enumerate() {
                    let value = sample? as f32 / max;
                    push_sample(&mut left, &mut right, channels, index, value);
                }
            }
        }
    }
    if channels == 1 {
        right.clone_from(&left);
    }
    Ok((left, right, spec.sample_rate))
}

fn push_sample(
    left: &mut Vec<f32>,
    right: &mut Vec<f32>,
    channels: usize,
    index: usize,
    value: f32,
) {
    if channels == 1 {
        left.push(value);
        return;
    }
    if index.is_multiple_of(2) {
        left.push(value);
    } else {
        right.push(value);
    }
}

fn resample_channel(samples: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input_rate == output_rate {
        return samples.to_vec();
    }
    let output_len = resample_len(samples.len(), input_rate, output_rate);
    let mut output = vec![0.0f32; output_len];
    linear_resample(samples, &mut output);
    output
}

fn resample_len(input_len: usize, input_rate: u32, output_rate: u32) -> usize {
    let input_len = input_len as u64;
    let input_rate = input_rate as u64;
    let output_rate = output_rate as u64;
    (input_len * output_rate).div_ceil(input_rate) as usize
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
