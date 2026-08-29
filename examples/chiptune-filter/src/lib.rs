use aviutl2::{
    AnyResult,
    filter::{
        FilterConfigItemSliceExt, FilterConfigItems, FilterPlugin, FilterPluginTable,
        FilterProcAudio,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, aviutl2::filter::FilterConfigSelectItems)]
enum WaveType {
    #[item(name = "矩形波")]
    Square,
    #[item(name = "三角波")]
    Triangle,
    #[item(name = "のこぎり波")]
    Sawtooth,
    #[item(name = "正弦波")]
    Sine,
    #[item(name = "ノイズ")]
    Noise,
}

#[derive(Debug, Clone, PartialEq, Eq, aviutl2::filter::FilterConfigSelectItems)]
enum FrequencyMode {
    #[item(name = "MIDIノート")]
    MidiNote,
    #[item(name = "周波数（Hz）")]
    FrequencyHz,
}

#[aviutl2::filter::filter_config_items]
#[derive(Debug, Clone)]
struct FilterConfig {
    #[track(name = "音量", range = 0.0..=1.0, step = 0.01, default = 0.5)]
    volume: f64,
    #[select(
        name = "音源",
        items = WaveType,
        default = WaveType::Square
    )]
    wave_type: WaveType,
    #[hide(wave_type == WaveType::Noise)]
    #[select(
        name = "周波数モード",
        items = FrequencyMode,
        default = FrequencyMode::MidiNote
    )]
    freq_mode: FrequencyMode,
    #[track(name = "MIDIノート", range = 0..=127, step = 1.0, default = 60)]
    #[hide(freq_mode != FrequencyMode::MidiNote)]
    #[hide(wave_type == WaveType::Noise)]
    midi_note: f64,
    #[track(name = "周波数（Hz）", range = 20.0..=20000.0, step = 1.0, default = 440.0)]
    #[hide(freq_mode != FrequencyMode::FrequencyHz)]
    #[hide(wave_type == WaveType::Noise)]
    frequency: f64,
}

#[aviutl2::plugin(FilterPlugin)]
struct ChiptuneFilter;

impl FilterPlugin for ChiptuneFilter {
    type Userdata = ();

    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(Self)
    }

    fn plugin_info(&self) -> FilterPluginTable {
        FilterPluginTable {
            name: "Rusty Chiptune Filter".to_string(),
            label: None,
            information: format!(
                "Example chiptune synthesizer, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/chiptune-filter",
                version = env!("CARGO_PKG_VERSION")
            ),
            flags: aviutl2::bitflag!(aviutl2::filter::FilterPluginFlags {
                audio: true,
                input: true,
            }),
            config_items: FilterConfig::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &mut FilterProcAudio<Self::Userdata>,
    ) -> AnyResult<()> {
        let config: FilterConfig = config.to_struct();

        let sample_rate = audio.scene.sample_rate as f64;
        let sample_num = audio.audio_object.sample_num as usize;
        let frequency = if config.freq_mode == FrequencyMode::MidiNote {
            440.0 * 2.0f64.powf((config.midi_note - 69.0) / 12.0)
        } else {
            config.frequency
        };

        let mut samples = vec![0.0; sample_num];
        let samples_per_cycle = sample_rate / frequency;

        for (sample_index, sample) in (audio.audio_object.sample_index..).zip(samples.iter_mut()) {
            let phase = (sample_index as f64 / samples_per_cycle) % 1.0;
            let value = match config.wave_type {
                WaveType::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                WaveType::Triangle => {
                    if phase < 0.5 {
                        phase * 4.0 - 1.0
                    } else {
                        (1.0 - phase) * 4.0 - 1.0
                    }
                }
                WaveType::Sawtooth => phase * 2.0 - 1.0,
                WaveType::Sine => (phase * 2.0 * std::f64::consts::PI).sin(),
                WaveType::Noise => rand::random::<f64>() * 2.0 - 1.0,
            };
            *sample = (value * config.volume) as f32;
        }

        for channel in 0..audio.audio_object.channel_num {
            audio.set_sample_data(aviutl2::filter::AudioChannel::Any(channel as i32), &samples);
        }

        Ok(())
    }
}

aviutl2::register_filter_plugin!(ChiptuneFilter);
