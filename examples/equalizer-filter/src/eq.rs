use biquad::{Biquad, ToHertz};

pub struct EqState {
    bass: PeakEq,
    mid: PeakEq,
    treble: PeakEq,
    lopass: LowPass,
    hipass: HighPass,

    wet: f32,
    lopass_enable: bool,
    hipass_enable: bool,
}
impl EqState {
    pub fn new(sample_rate: f32, config: &crate::FilterConfig) -> Self {
        Self {
            bass: PeakEq::new(config.bass_freq, config.bass_gain, sample_rate),
            mid: PeakEq::new(config.mid_freq, config.mid_gain, sample_rate),
            treble: PeakEq::new(config.treble_freq, config.treble_gain, sample_rate),
            lopass: LowPass::new(config.lopass_freq, sample_rate),
            hipass: HighPass::new(config.hipass_freq, sample_rate),

            wet: config.wet,
            lopass_enable: config.lopass_enable,
            hipass_enable: config.hipass_enable,
        }
    }

    pub fn update_params(&mut self, sample_rate: f32, config: &crate::FilterConfig) {
        self.bass
            .set_params(config.bass_freq, config.bass_gain, sample_rate);
        self.mid
            .set_params(config.mid_freq, config.mid_gain, sample_rate);
        self.treble
            .set_params(config.treble_freq, config.treble_gain, sample_rate);
        self.lopass.set_params(config.lopass_freq, sample_rate);
        self.hipass.set_params(config.hipass_freq, sample_rate);

        self.wet = config.wet;
        self.lopass_enable = config.lopass_enable;
        self.hipass_enable = config.hipass_enable;
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            let mut s = *sample;
            let orig = s;
            if self.bass.gain != 0.0 {
                s = self.bass.apply(s);
            }
            if self.mid.gain != 0.0 {
                s = self.mid.apply(s);
            }
            if self.treble.gain != 0.0 {
                s = self.treble.apply(s);
            }
            if self.lopass_enable {
                s = self.lopass.apply(s);
            }
            if self.hipass_enable {
                s = self.hipass.apply(s);
            }
            *sample = s * self.wet + orig * (1.0 - self.wet);
        }
    }

    pub fn reset(&mut self) {
        self.bass.filter.reset_state();
        self.mid.filter.reset_state();
        self.treble.filter.reset_state();
        self.lopass.filter.reset_state();
        self.hipass.filter.reset_state();
    }
}
pub struct PeakEq {
    sample_rate: f32,
    freq: f32,
    gain: f32,

    filter: biquad::DirectForm1<f32>,
}
impl PeakEq {
    fn new(freq: f32, gain: f32, sample_rate: f32) -> Self {
        let coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::PeakingEQ(gain),
            sample_rate.hz(),
            freq.hz(),
            Q,
        )
        .unwrap();
        let filter = biquad::DirectForm1::<f32>::new(coeffs);
        Self {
            freq,
            gain,
            sample_rate,
            filter,
        }
    }

    fn set_params(&mut self, freq: f32, gain: f32, sample_rate: f32) {
        let coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::PeakingEQ(gain),
            sample_rate.hz(),
            freq.hz(),
            Q,
        )
        .unwrap();
        self.filter.update_coefficients(coeffs);
        self.freq = freq;
        self.gain = gain;
        self.sample_rate = sample_rate;
    }

    fn apply(&mut self, sample: f32) -> f32 {
        self.filter.run(sample)
    }
}
pub struct LowPass {
    sample_rate: f32,
    freq: f32,
    filter: biquad::DirectForm1<f32>,
}
impl LowPass {
    fn new(freq: f32, sample_rate: f32) -> Self {
        let coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::LowPass,
            sample_rate.hz(),
            freq.hz(),
            Q,
        )
        .unwrap();
        let filter = biquad::DirectForm1::<f32>::new(coeffs);
        Self {
            freq,
            sample_rate,
            filter,
        }
    }

    fn set_params(&mut self, freq: f32, sample_rate: f32) {
        let coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::LowPass,
            sample_rate.hz(),
            freq.hz(),
            Q,
        )
        .unwrap();
        self.filter.update_coefficients(coeffs);
        self.freq = freq;
        self.sample_rate = sample_rate;
    }

    fn apply(&mut self, sample: f32) -> f32 {
        self.filter.run(sample)
    }
}
pub struct HighPass {
    sample_rate: f32,
    freq: f32,
    filter: biquad::DirectForm1<f32>,
}
impl HighPass {
    fn new(freq: f32, sample_rate: f32) -> Self {
        let coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::HighPass,
            sample_rate.hz(),
            freq.hz(),
            Q,
        )
        .unwrap();
        let filter = biquad::DirectForm1::<f32>::new(coeffs);
        Self {
            freq,
            sample_rate,
            filter,
        }
    }

    fn set_params(&mut self, freq: f32, sample_rate: f32) {
        let coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::HighPass,
            sample_rate.hz(),
            freq.hz(),
            Q,
        )
        .unwrap();
        self.filter.update_coefficients(coeffs);
        self.freq = freq;
        self.sample_rate = sample_rate;
    }

    fn apply(&mut self, sample: f32) -> f32 {
        self.filter.run(sample)
    }
}

pub const Q: f32 = std::f32::consts::FRAC_1_SQRT_2; // Quality factor for the filters
