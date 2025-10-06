mod eq;
use aviutl2::{
    filter::{FilterConfigItemSliceExt, FilterConfigItems},
    log,
};

#[derive(Debug, Clone, aviutl2::filter::FilterConfigItems)]
pub struct FilterConfig {
    #[track(name = "Wet", range = 0.0..=1.0, step = 0.01, default = 1.0)]
    wet: f32,
    #[track(name = "Bass: Gain", range = -15.0..=15.0, step = 0.1, default = 0.0)]
    bass_gain: f32,
    #[track(name = "Mid: Gain", range = -15.0..=15.0, step = 0.1, default = 0.0)]
    mid_gain: f32,
    #[track(name = "Treble: Gain", range = -15.0..=15.0, step = 0.1, default = 0.0)]
    treble_gain: f32,
    #[track(name = "Bass: Frequency", range = 20.0..=250.0, step = 1.0, default = 100.0)]
    bass_freq: f32,
    #[track(name = "Mid: Frequency", range = 250.0..=4000.0, step = 1.0, default = 1000.0)]
    mid_freq: f32,
    #[track(name = "Treble: Frequency", range = 4000.0..=20000.0, step = 1.0, default = 10000.0)]
    treble_freq: f32,

    #[check(name = "Hi-pass: Enable", default = false)]
    hipass_enable: bool,
    #[track(name = "Hi-pass: Frequency", range = 20.0..=20000.0, step = 1.0, default = 20.0)]
    hipass_freq: f32,
    #[check(name = "Lo-pass: Enable", default = false)]
    lopass_enable: bool,
    #[track(name = "Lo-pass: Frequency", range = 20.0..=20000.0, step = 1.0, default = 20000.0)]
    lopass_freq: f32,
}

struct EqualizerFilter {
    #[expect(clippy::type_complexity)]
    q_states: std::sync::RwLock<
        std::collections::HashMap<
            i64,
            std::sync::Arc<std::sync::Mutex<(eq::EqState, eq::EqState)>>,
        >,
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
            name: "Equalizer Filter".to_string(),
            label: None,
            information: "An example equalizer filter plugin.".to_string(),
            filter_type: aviutl2::filter::FilterType::Both,
            wants_initial_input: false,
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &aviutl2::filter::FilterProcAudio,
    ) -> anyhow::Result<()> {
        let config: FilterConfig = config.to_struct();
        if config.wet == 0.0 {
            return Ok(());
        }

        let mut left_samples = audio.get_sample_data(0);
        let mut right_samples = audio.get_sample_data(1);
        let sample_rate = audio.scene.sample_rate as f32;
        let obj_id = audio.object.id;

        let q_state = {
            let q_states = self.q_states.read().unwrap();
            q_states.get(&obj_id).cloned()
        };
        let q_state = if let Some(q_state) = q_state {
            q_state
        } else {
            log::info!("Creating new EQ state for object ID {}", obj_id);
            let new_state = std::sync::Arc::new(std::sync::Mutex::new((
                eq::EqState::new(sample_rate, &config),
                eq::EqState::new(sample_rate, &config),
            )));
            let mut q_states = self.q_states.write().unwrap();
            q_states.insert(obj_id, new_state.clone());
            new_state
        };

        {
            let (left_state, right_state) = &mut *q_state.lock().unwrap();
            left_state.update_params(sample_rate, &config);
            right_state.update_params(sample_rate, &config);

            left_state.process(&mut left_samples);
            right_state.process(&mut right_samples);
        }
        audio.set_sample_data(&left_samples, 0);
        audio.set_sample_data(&right_samples, 1);

        Ok(())
    }
}

aviutl2::register_filter_plugin!(EqualizerFilter);
