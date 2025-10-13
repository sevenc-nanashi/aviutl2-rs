mod eq;
use aviutl2::{
    filter::{FilterConfigItemSliceExt, FilterConfigItems},
    log,
};

#[derive(Debug, Clone, PartialEq, aviutl2::filter::FilterConfigItems)]
pub struct FilterConfig {
    #[track(name = "Wet", range = 0.0..=1.0, step = 0.01, default = 1.0)]
    wet: f64,
    #[track(name = "Bass: Gain", range = -15.0..=15.0, step = 0.1, default = 0.0)]
    bass_gain: f64,
    #[track(name = "Mid: Gain", range = -15.0..=15.0, step = 0.1, default = 0.0)]
    mid_gain: f64,
    #[track(name = "Treble: Gain", range = -15.0..=15.0, step = 0.1, default = 0.0)]
    treble_gain: f64,
    #[track(name = "Bass: Frequency", range = 20.0..=250.0, step = 1.0, default = 100.0)]
    bass_freq: f64,
    #[track(name = "Mid: Frequency", range = 250.0..=4000.0, step = 1.0, default = 1000.0)]
    mid_freq: f64,
    #[track(name = "Treble: Frequency", range = 4000.0..=20000.0, step = 1.0, default = 10000.0)]
    treble_freq: f64,

    #[check(name = "Hi-pass: Enable", default = false)]
    hipass_enable: bool,
    #[track(name = "Hi-pass: Frequency", range = 20.0..=20000.0, step = 1.0, default = 20.0)]
    hipass_freq: f64,
    #[check(name = "Lo-pass: Enable", default = false)]
    lopass_enable: bool,
    #[track(name = "Lo-pass: Frequency", range = 20.0..=20000.0, step = 1.0, default = 20000.0)]
    lopass_freq: f64,
}

const NUM_CACHES: usize = 2;
struct EqStates {
    left: eq::EqState,
    right: eq::EqState,

    expected_next_index: u64,
    next_cache_index: usize,
    caches: Vec<EqCache>,
}
struct EqCache {
    sample_index: u64,
    config: FilterConfig,
    left: Vec<f32>,
    right: Vec<f32>,
}
impl EqStates {
    fn new(sample_rate: f64, config: &FilterConfig) -> Self {
        Self {
            left: eq::EqState::new(sample_rate, config),
            right: eq::EqState::new(sample_rate, config),
            expected_next_index: 0,
            next_cache_index: 0,
            caches: (0..NUM_CACHES)
                .map(|_| EqCache {
                    sample_index: u64::MAX,
                    config: config.clone(),
                    left: Vec::new(),
                    right: Vec::new(),
                })
                .collect(),
        }
    }
    fn update_params(&mut self, sample_rate: f64, config: &FilterConfig) {
        self.left.update_params(sample_rate, config);
        self.right.update_params(sample_rate, config);
    }
    fn process(&mut self, left: &mut [f64], right: &mut [f64]) {
        self.left.process(left);
        self.right.process(right);
    }
    fn reset(&mut self) {
        self.left.reset();
        self.right.reset();
    }
}

struct EqualizerFilter {
    q_states: std::sync::RwLock<
        std::collections::HashMap<i64, std::sync::Arc<std::sync::Mutex<EqStates>>>,
    >,
}

impl aviutl2::filter::FilterPlugin for EqualizerFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        env_logger::Builder::new()
            .parse_filters("info")
            .target(aviutl2::utils::debug_logger_target())
            .init();
        Ok(Self {
            q_states: std::sync::RwLock::new(std::collections::HashMap::new()),
        })
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Rusty Equalizer Filter".to_string(),
            label: None,
            information: format!(
                "Simple equalizer, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/equalizer-filter",
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

        let left_samples = audio.get_sample_data(aviutl2::filter::AudioChannel::Left);
        let right_samples = audio.get_sample_data(aviutl2::filter::AudioChannel::Right);
        let sample_rate = audio.scene.sample_rate as f64;
        let obj_id = audio.object.id;

        let q_state = {
            let q_states = self.q_states.read().unwrap();
            q_states.get(&obj_id).cloned()
        };
        let q_state = if let Some(q_state) = q_state {
            q_state
        } else {
            log::info!("Creating new EQ state for object ID {}", obj_id);
            let new_state =
                std::sync::Arc::new(std::sync::Mutex::new(EqStates::new(sample_rate, &config)));
            let mut q_states = self.q_states.write().unwrap();
            q_states.insert(obj_id, new_state.clone());
            new_state
        };

        {
            let mut q_state = q_state.lock().unwrap();
            for cache in &mut q_state.caches {
                if cache.sample_index == audio.audio_object.sample_index
                    && cache.config == config
                    && cache.left.len() == left_samples.len()
                    && cache.right.len() == right_samples.len()
                {
                    log::debug!(
                        "Using cached EQ result for object ID {} at sample_index {}",
                        obj_id,
                        audio.audio_object.sample_index
                    );
                    audio.set_sample_data(&cache.left, aviutl2::filter::AudioChannel::Left);
                    audio.set_sample_data(&cache.right, aviutl2::filter::AudioChannel::Right);
                    return Ok(());
                }
            }
            if q_state.expected_next_index != audio.audio_object.sample_index {
                log::debug!(
                    "Audio discontinuity detected for object ID {}: expected {}, got {}",
                    obj_id,
                    q_state.expected_next_index,
                    audio.audio_object.sample_index
                );
                q_state.reset();
            }
            log::debug!(
                "Processing audio for object ID {}: sample_index {}, num_samples {}",
                obj_id,
                audio.audio_object.sample_index,
                left_samples.len()
            );
            q_state.expected_next_index =
                audio.audio_object.sample_index + left_samples.len() as u64;

            q_state.update_params(sample_rate, &config);

            let mut left_samples = left_samples
                .into_iter()
                .map(|s| s as f64)
                .collect::<Vec<_>>();
            let mut right_samples = right_samples
                .into_iter()
                .map(|s| s as f64)
                .collect::<Vec<_>>();
            q_state.process(&mut left_samples, &mut right_samples);
            let next_cache_index = q_state.next_cache_index;
            let left_samples = left_samples.iter().map(|&s| s as f32).collect::<Vec<_>>();
            let right_samples = right_samples.iter().map(|&s| s as f32).collect::<Vec<_>>();
            audio.set_sample_data(&left_samples, aviutl2::filter::AudioChannel::Left);
            audio.set_sample_data(&right_samples, aviutl2::filter::AudioChannel::Right);

            let cache = &mut q_state.caches[next_cache_index];
            cache.sample_index = audio.audio_object.sample_index;
            cache.config = config.clone();
            cache.left.clear();
            cache.left.extend_from_slice(&left_samples);
            cache.right.clear();
            cache.right.extend_from_slice(&right_samples);
            q_state.next_cache_index = (q_state.next_cache_index + 1) % NUM_CACHES;
        }

        Ok(())
    }
}

aviutl2::register_filter_plugin!(EqualizerFilter);
