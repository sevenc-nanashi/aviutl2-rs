use std::num::NonZeroIsize;

use crate::{
    common::{
        alert_error, free_leaked_memory, leak_large_string, load_large_string,
        result_to_bool_with_dialog,
    },
    output::{
        AudioOutputInfo, FromRawAudioSamples, FromRawVideoFrame, OutputInfo, OutputPlugin,
        RgbVideoFrame, VideoOutputInfo,
    },
};

use aviutl2_sys::{
    input2::WAVE_FORMAT_PCM,
    output2::{BI_RGB, LPCWSTR, WAVE_FORMAT_IEEE_FLOAT},
};
use num_rational::Rational32;

use crate::common::format_file_filters;

use super::{RawBgrVideoFrame, RawYuy2VideoFrame, Yuy2VideoFrame};

impl FromRawVideoFrame for RgbVideoFrame {
    const FORMAT: u32 = BI_RGB;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 3 bytes (BGR)
                let pixel_r = unsafe { *frame_data_ptr.add(i * 3 + 2) };
                let pixel_g = unsafe { *frame_data_ptr.add(i * 3 + 1) };
                let pixel_b = unsafe { *frame_data_ptr.add(i * 3) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((pixel_r, pixel_g, pixel_b));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for Yuy2VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_YUY2;

    fn check(video: &VideoOutputInfo) -> Result<(), String> {
        if video.width % 2 != 0 || video.height % 2 != 0 {
            return Err("YUY2 format requires even width and height".to_string());
        }
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height / 2) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        for y in 0..video.height as usize {
            for x in 0..(video.width / 2) as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 4 bytes (YUY2)
                let d_y1 = unsafe { *frame_data_ptr.add(i * 4) };
                let d_u = unsafe { *frame_data_ptr.add(i * 4 + 1) };
                let d_y2 = unsafe { *frame_data_ptr.add(i * 4 + 2) };
                let d_v = unsafe { *frame_data_ptr.add(i * 4 + 3) };

                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((d_y1, d_u, d_y2, d_v));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height * 2) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for RawBgrVideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_RGB;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let frame_buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 3) as usize)
                .to_owned()
        };

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for RawYuy2VideoFrame {
    const FORMAT: u32 = aviutl2_sys::output2::BI_YUY2;

    fn check(video: &VideoOutputInfo) -> Result<(), String> {
        if video.width % 2 != 0 || video.height % 2 != 0 {
            return Err("YUY2 format requires even width and height".to_string());
        }
        Ok(())
    }
    unsafe fn from_raw(video: &VideoOutputInfo, frame_data_ptr: *const u8) -> Self {
        let frame_buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 2) as usize)
                .to_owned()
        };

        Self { data: frame_buffer }
    }
}

impl FromRawAudioSamples for f32 {
    const FORMAT: u32 = WAVE_FORMAT_IEEE_FLOAT;

    unsafe fn from_raw(length: i32, num_channels: u32, frame_data_ptr: *const u8) -> Vec<Self> {
        let frame_data_slice = unsafe {
            std::slice::from_raw_parts(
                frame_data_ptr as *const f32,
                length as usize * num_channels as usize,
            )
        };
        frame_data_slice.to_vec()
    }
}
impl FromRawAudioSamples for i16 {
    const FORMAT: u32 = WAVE_FORMAT_PCM;

    unsafe fn from_raw(length: i32, num_channels: u32, frame_data_ptr: *const u8) -> Vec<Self> {
        let frame_data_slice = unsafe {
            std::slice::from_raw_parts(
                frame_data_ptr as *const i16,
                length as usize * num_channels as usize,
            )
        };
        frame_data_slice.to_vec()
    }
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

    pub fn get_audio_samples<F: FromRawAudioSamples>(&self, start: i32, length: i32) -> Option<(Vec<F>, u32)> {
        let audio_ptr = unsafe { self.internal.as_mut().and_then(|oip| oip.func_get_audio) }?;
        let mut readed = 0;
        let audio_data_ptr = audio_ptr(start, length, &mut readed, F::FORMAT) as *mut u8;

        let audio = self.audio.as_ref()?;
        let samples = unsafe {
            F::from_raw(length, audio.num_channels, audio_data_ptr)
        };

        Some((samples, audio.num_channels))
    }

    pub fn get_mono_audio_samples<F: FromRawAudioSamples>(&self, start: i32, length: i32) -> Option<Vec<F>> {
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

    pub fn get_mono_audio_samples_iter<F: FromRawAudioSamples>(&self, length: i32) -> MonoAudioSamplesIterator<F> {
        MonoAudioSamplesIterator::new(self, length)
    }

    pub fn get_stereo_audio_samples<F: FromRawAudioSamples>(&self, start: i32, length: i32) -> Option<Vec<(F, F)>> {
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

    pub fn get_stereo_audio_samples_iter<F: FromRawAudioSamples>(&self, length: i32) -> StereoAudioSamplesIterator<F> {
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

pub unsafe fn create_table<T: OutputPlugin>(
    plugin: &T,
    func_output: extern "C" fn(*mut aviutl2_sys::output2::OUTPUT_INFO) -> bool,
    func_config: extern "C" fn(aviutl2_sys::output2::HWND, aviutl2_sys::output2::HINSTANCE) -> bool,
    func_get_config_text: extern "C" fn() -> LPCWSTR,
) -> aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    free_leaked_memory();
    let plugin_info = plugin.plugin_info();
    let filefilter = format_file_filters(&plugin_info.file_filters);
    let filefilter = leak_large_string(&filefilter);

    let name = if cfg!(debug_assertions) {
        format!("{} (Debug)", plugin_info.name)
    } else {
        plugin_info.name
    };
    let information = if cfg!(debug_assertions) {
        format!("{} (Debug Build)", plugin_info.information)
    } else {
        plugin_info.information
    };

    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
        flag: plugin_info.input_type.to_bits(),
        name: leak_large_string(&name),
        filefilter,
        information: leak_large_string(&information),
        func_output: Some(func_output),
        func_config: plugin_info.can_config.then_some(func_config),
        func_get_config_text: Some(func_get_config_text),
    }
}

pub unsafe fn func_output<T: OutputPlugin>(
    plugin: &T,
    oip: *mut aviutl2_sys::output2::OUTPUT_INFO,
) -> bool {
    let oip = unsafe { &mut *oip };
    let output_info = OutputInfo::from_raw(oip);
    result_to_bool_with_dialog(plugin.output(output_info))
}

pub unsafe fn func_config<T: OutputPlugin>(
    plugin: &T,
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    let mut handle =
        raw_window_handle::Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
    handle.hinstance = Some(NonZeroIsize::new(dll_hinst as isize).unwrap());
    result_to_bool_with_dialog(plugin.config(handle))
}

pub unsafe fn func_get_config_text<T: OutputPlugin>(plugin: &T) -> *mut u16 {
    let text = plugin.config_text();
    match text {
        Ok(text) => leak_large_string(&text),
        Err(e) => {
            alert_error(&e);
            leak_large_string("Error")
        }
    }
}

#[macro_export]
macro_rules! register_output_plugin {
    ($struct:ident) => {
        #[doc(hidden)]
        mod __au2_register_output_plugin {
            use super::*;

            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

            #[unsafe(no_mangle)]
            unsafe extern "C" fn GetOutputPluginTable()
            -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE {
                let table = unsafe {
                    $crate::output::__bridge::create_table::<$struct>(
                        &*PLUGIN,
                        func_output,
                        func_config,
                        func_get_config_text,
                    )
                };
                Box::into_raw(Box::new(table))
            }

            extern "C" fn func_output(oip: *mut aviutl2::sys::output2::OUTPUT_INFO) -> bool {
                unsafe { $crate::output::__bridge::func_output(&*PLUGIN, oip) }
            }

            extern "C" fn func_config(
                hwnd: aviutl2::sys::output2::HWND,
                dll_hinst: aviutl2::sys::output2::HINSTANCE,
            ) -> bool {
                unsafe { $crate::output::__bridge::func_config(&*PLUGIN, hwnd, dll_hinst) }
            }

            extern "C" fn func_get_config_text() -> *const u16 {
                unsafe { $crate::output::__bridge::func_get_config_text(&*PLUGIN) }
            }
        }
    };
}
