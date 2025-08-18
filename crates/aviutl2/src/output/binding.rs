use crate::common::{AnyResult, FileFilter, load_large_string};
use aviutl2_sys::output2::OUTPUT_INFO;
use std::ops::Deref;

pub use half::f16;
pub use num_rational::Rational32;
pub use raw_window_handle::Win32WindowHandle;

pub struct OutputPluginTable {
    pub name: String,
    pub input_type: OutputType,
    pub file_filters: Vec<FileFilter>,
    pub information: String,

    pub can_config: bool,
}

pub enum OutputType {
    Video,
    Audio,
    Both,
}

impl OutputType {
    pub fn to_bits(&self) -> i32 {
        match self {
            OutputType::Video => 1,
            OutputType::Audio => 2,
            OutputType::Both => 3,
        }
    }
}

pub struct OutputInfo {
    pub video: Option<VideoOutputInfo>,
    pub audio: Option<AudioOutputInfo>,
    pub path: std::path::PathBuf,

    pub(crate) internal: *mut OUTPUT_INFO,
}

unsafe impl Send for OutputInfo {}
unsafe impl Sync for OutputInfo {}

pub struct VideoOutputInfo {
    pub width: u32,
    pub height: u32,
    pub fps: Rational32,
    pub num_frames: u32,
}

pub struct AudioOutputInfo {
    pub sample_rate: u32,
    pub num_samples: u32,
    pub num_channels: u32,
}

pub trait OutputPlugin: Send + Sync {
    fn new() -> Self;

    fn plugin_info(&self) -> OutputPluginTable;

    fn output(&self, info: OutputInfo) -> AnyResult<()>;

    fn config(&self, _hwnd: Win32WindowHandle) -> AnyResult<()> {
        Ok(())
    }

    fn config_text(&self) -> AnyResult<String> {
        Ok(String::new())
    }
}

pub trait FromRawVideoFrame {
    const FORMAT: u32;

    fn check(video: &VideoOutputInfo) -> Result<(), String>;
    /// # Safety
    /// func_get_videoの戻り値のポインタのみが許容される。
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self;
}

duplicate::duplicate! {
    [
        Name                Type;
        [RgbVideoFrame]     [(u8, u8, u8)];
        [Yuy2VideoFrame]    [(u8, u8, u8, u8)];
        [Hf64VideoFrame]    [(f16, f16, f16, f16)];
        [Yc48VideoFrame]    [(u16, u16, u16)];
        [Pa64VideoFrame]    [(u16, u16, u16, u16)];

        [RawBgrVideoFrame]  [u8];
        [RawYuy2VideoFrame] [u8];
        [RawHf64VideoFrame] [u16];
        [RawYc48VideoFrame] [u16];
        [RawPa64VideoFrame] [u16];
    ]
    #[derive(Debug, Clone)]
    pub struct Name {
        pub data: Vec<Type>, // Raw RGB format
    }
    impl Deref for Name {
        type Target = [Type];

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }
}

pub trait FromRawAudioSamples: Sized + Send + Sync + Copy {
    const FORMAT: u32;

    /// # Safety
    /// func_get_audioの戻り値のポインタのみが許容される。
    unsafe fn from_raw(length: i32, num_channels: u32, audio_data_ptr: *const u8) -> Vec<Self>;
}

impl OutputInfo {
    pub(crate) fn from_raw(oip: *mut aviutl2_sys::output2::OUTPUT_INFO) -> Self {
        let raw = unsafe { &*oip };

        Self {
            video: if raw.flag & aviutl2_sys::output2::OUTPUT_INFO::FLAG_VIDEO != 0 {
                Some(VideoOutputInfo {
                    width: raw.w as u32,
                    height: raw.h as u32,
                    fps: Rational32::new(raw.rate, raw.scale),
                    num_frames: raw.n as u32,
                })
            } else {
                None
            },
            audio: if raw.flag & aviutl2_sys::output2::OUTPUT_INFO::FLAG_AUDIO != 0 {
                Some(AudioOutputInfo {
                    sample_rate: raw.audio_rate as u32,
                    num_samples: raw.audio_n as u32,
                    num_channels: raw.audio_ch as u32,
                })
            } else {
                None
            },

            path: std::path::PathBuf::from(load_large_string(raw.savefile)),

            internal: oip,
        }
    }

    pub fn get_video_frame<F: FromRawVideoFrame>(&self, frame: i32) -> Option<F> {
        if let Some(video) = &self.video {
            if F::check(video).is_err() {
                return None;
            }
            unsafe { self.get_video_frame_unchecked::<F>(frame) }
        } else {
            None
        }
    }

    pub unsafe fn get_video_frame_unchecked<F: FromRawVideoFrame>(&self, frame: i32) -> Option<F> {
        let frame_ptr = unsafe { self.internal.as_mut().and_then(|oip| oip.func_get_video) }?;
        let frame_data_ptr = frame_ptr(frame, F::FORMAT) as *mut u8;
        let video = self.video.as_ref()?;
        let frame = unsafe { F::from_raw(video, frame_data_ptr) };
        Some(frame)
    }

    pub fn get_video_frames_iter<F: FromRawVideoFrame>(&self) -> VideoFramesIterator<'_, F> {
        VideoFramesIterator::new(self)
    }

    pub fn get_audio_samples<F: FromRawAudioSamples>(
        &self,
        start: i32,
        length: i32,
    ) -> Option<(Vec<F>, u32)> {
        let audio_ptr = unsafe { self.internal.as_mut().and_then(|oip| oip.func_get_audio) }?;
        let mut readed = 0;
        let audio_data_ptr = audio_ptr(start, length, &mut readed, F::FORMAT) as *mut u8;

        let audio = self.audio.as_ref()?;
        let samples = unsafe { F::from_raw(length, audio.num_channels, audio_data_ptr) };

        Some((samples, audio.num_channels))
    }

    pub fn get_mono_audio_samples<F: FromRawAudioSamples>(
        &self,
        start: i32,
        length: i32,
    ) -> Option<Vec<F>> {
        let (samples, num_channels) = self.get_audio_samples(start, length)?;
        if num_channels == 1 {
            Some(samples)
        } else {
            Some(
                samples
                    .chunks(num_channels as usize)
                    .map(|chunk| chunk[0])
                    .collect(),
            )
        }
    }

    pub fn get_mono_audio_samples_iter<F: FromRawAudioSamples>(
        &self,
        length: i32,
    ) -> MonoAudioSamplesIterator<F> {
        MonoAudioSamplesIterator::new(self, length)
    }

    pub fn get_stereo_audio_samples<F: FromRawAudioSamples>(
        &self,
        start: i32,
        length: i32,
    ) -> Option<Vec<(F, F)>> {
        let (samples, num_channels) = self.get_audio_samples(start, length)?;
        if num_channels == 2 {
            Some(
                samples
                    .chunks(num_channels as usize)
                    .map(|chunk| (chunk[0], chunk[1]))
                    .collect(),
            )
        } else {
            None
        }
    }

    pub fn get_stereo_audio_samples_iter<F: FromRawAudioSamples>(
        &self,
        length: i32,
    ) -> StereoAudioSamplesIterator<F> {
        StereoAudioSamplesIterator::new(self, length)
    }

    pub fn is_aborted(&self) -> bool {
        let is_abort_func = unsafe { self.internal.as_mut().and_then(|oip| oip.func_is_abort) };
        is_abort_func.is_none_or(|f| f())
    }

    pub fn update_display(&self, current_frame: i32, total_frames: i32) {
        if let Some(func) = unsafe {
            self.internal
                .as_mut()
                .and_then(|oip| oip.func_rest_time_disp)
        } {
            func(current_frame, total_frames);
        }
    }

    pub fn set_buffer_size(&self, video_size: i32, audio_size: i32) {
        if let Some(func) = unsafe {
            self.internal
                .as_mut()
                .and_then(|oip| oip.func_set_buffer_size)
        } {
            func(video_size, audio_size);
        }
    }
}

pub struct VideoFramesIterator<'a, F: FromRawVideoFrame> {
    output_info: &'a OutputInfo,
    current_frame: i32,
    total_frames: i32,
    last_updated_time: std::time::Instant,
    check_result: bool,
    _marker: std::marker::PhantomData<F>,
}

impl<'a, F: FromRawVideoFrame> VideoFramesIterator<'a, F> {
    pub fn new(output_info: &'a OutputInfo) -> Self {
        let total_frames = output_info
            .video
            .as_ref()
            .map_or(0, |v| v.num_frames as i32);
        Self {
            output_info,
            current_frame: 0,
            total_frames,
            last_updated_time: std::time::Instant::now(),
            check_result: output_info
                .video
                .as_ref()
                .is_some_and(|v| F::check(v).is_ok()),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, F: FromRawVideoFrame> Iterator for VideoFramesIterator<'a, F> {
    type Item = (i32, F);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.check_result {
            return None;
        }
        if self.current_frame >= self.total_frames {
            return None;
        }

        if self.output_info.is_aborted() {
            return None;
        }

        let frame = unsafe {
            self.output_info
                .get_video_frame_unchecked(self.current_frame)
        };
        if let Some(frame_data) = frame {
            let current_frame = self.current_frame;
            self.current_frame += 1;
            if self.last_updated_time.elapsed().as_secs_f32() > 0.1 {
                self.output_info
                    .update_display(current_frame, self.total_frames);
                self.last_updated_time = std::time::Instant::now();
            }
            Some((current_frame, frame_data))
        } else {
            None
        }
    }
}

duplicate::duplicate! {
    [
        Name method IterType;
        [MonoAudioSamplesIterator] [get_mono_audio_samples] [F];
        [StereoAudioSamplesIterator] [get_stereo_audio_samples] [(F, F)];
    ]

    pub struct Name<'a, F: FromRawAudioSamples> {
        output_info: &'a OutputInfo,
        length: i32,
        total_length: i32,
        readed: i32,
        _marker: std::marker::PhantomData<F>,
    }

    impl<'a, F: FromRawAudioSamples> Name<'a, F> {
        pub fn new(output_info: &'a OutputInfo, length: i32) -> Self {
            Self {
                output_info,
                length,
                total_length: output_info.audio.as_ref().map_or(0, |a| a.num_samples as i32),
                readed: 0,
                _marker: std::marker::PhantomData,
            }
        }
    }

    impl<'a, F: FromRawAudioSamples> Iterator for Name<'a, F> {
        type Item = (usize, Vec<IterType>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.readed >= self.total_length {
                return None;
            }
            if self.output_info.is_aborted() {
                return None;
            }

            let length_to_read = self.length.min(self.total_length - self.readed);
            let samples = self.output_info.method(self.readed, length_to_read);
            if let Some(samples) = samples {
                let start_frame = self.readed;
                self.readed += samples.len() as i32;
                Some((start_frame as usize, samples))
            } else {
                None
            }
        }
    }
}
