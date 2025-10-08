use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcAudio,
    },
};
use rand::Rng;

#[derive(Debug, Clone, PartialEq, FilterConfigItems)]
struct FilterConfig {
    #[track(name = "Volume", range = 0.0..=1.0, step = 0.01, default = 0.5)]
    volume: f64,
    #[select(
        name = "Wave Type",
        items = ["Square", "Triangle", "Sawtooth", "Sine", "Noise"],
        default = 0
    )]
    wave_type: usize,
    #[select(
        name = "Frequency Mode",
        items = ["MIDI Note", "Frequency (Hz)"],
        default = 0
    )]
    freq_mode: usize,
    #[track(name = "MIDI Note", range = 0.0..=127.0, step = 1.0, default = 69.0)]
    midi_note: f64,
    #[track(name = "Frequency", range = 20.0..=20000.0, step = 1.0, default = 440.0)]
    frequency: f64,
}

struct Synthesizer {
    phase: f64,
}
impl Synthesizer {
    fn new() -> Self {
        Self { phase: 0.0 }
    }
}

struct ChiptuneFilter {
    synthesizers: std::sync::RwLock<
        std::collections::HashMap<i64, std::sync::Arc<std::sync::Mutex<Synthesizer>>>,
    >,
}

impl FilterPlugin for ChiptuneFilter {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(Self {
            synthesizers: std::sync::RwLock::new(std::collections::HashMap::new()),
        })
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "Chiptune Synthesizer Filter".to_string(),
            label: None,
            information: format!(
                "Example chiptune synthesizer, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/chiptune-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            filter_type: aviutl2::filter::FilterType::Audio,
            as_object: true,
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &FilterProcAudio,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();

        let synthesizer = {
            let synthesizers = self.synthesizers.read().unwrap();
            synthesizers.get(&audio.object.id).cloned()
        };
        let synthesizer = if let Some(synthesizer) = synthesizer {
            synthesizer
        } else {
            let new_synthesizer = std::sync::Arc::new(std::sync::Mutex::new(Synthesizer::new()));
            let mut synthesizers = self.synthesizers.write().unwrap();
            synthesizers.insert(audio.object.id, new_synthesizer.clone());
            new_synthesizer
        };

        let mut synthesizer = synthesizer.lock().unwrap();

        let sample_rate = audio.scene.sample_rate as f64;
        let sample_num = audio.audio_object.sample_num as usize;
        let frequency = if config.freq_mode == 0 {
            440.0 * 2.0f64.powf((config.midi_note - 69.0) / 12.0)
        } else {
            config.frequency
        };

        let mut left = vec![0.0; sample_num];
        let mut right = vec![0.0; sample_num];

        let mut phase = synthesizer.phase;
        let mut rng = rand::rng();
        for i in 0..sample_num {
            let value = match config.wave_type {
                0 => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                1 => {
                    if phase < 0.5 {
                        phase * 4.0 - 1.0
                    } else {
                        (1.0 - phase) * 4.0 - 1.0
                    }
                }
                2 => phase * 2.0 - 1.0,
                3 => (phase * 2.0 * std::f64::consts::PI).sin(),
                4 => Rng::random::<f64>(&mut rng) * 2.0 - 1.0,
                _ => 0.0,
            };
            left[i] = (value * config.volume) as f32;
            right[i] = (value * config.volume) as f32;

            phase += frequency / sample_rate;
            if phase >= 1.0 {
                phase -= 1.0;
            }
        }

        synthesizer.phase = phase;

        audio.set_sample_data(&left, 0);
        audio.set_sample_data(&right, 1);

        Ok(())
    }
}

aviutl2::register_filter_plugin!(ChiptuneFilter);
