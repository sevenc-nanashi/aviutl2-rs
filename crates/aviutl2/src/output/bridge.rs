use crate::{
    common::{
        alert_error, free_leaked_memory, leak_large_string, load_large_string,
        result_to_bool_with_dialog,
    },
    output::{AudioOutputInfo, OutputInfo, OutputPlugin, VideoOutputInfo},
};

use aviutl2_sys::output2::{BI_RGB, LPCWSTR};
use num_rational::Rational32;

use crate::common::format_file_filters;

pub type VideoFrame = Vec<(u8, u8, u8)>; // RGB format

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

    pub fn get_video_frame(&self, frame: i32) -> Option<VideoFrame> {
        let frame_ptr = unsafe {
            (self.internal as *mut aviutl2_sys::output2::OUTPUT_INFO)
                .as_mut()
                .and_then(|oip| oip.func_get_video)
        }?;
        let frame_data_ptr = frame_ptr(frame, BI_RGB) as *mut u8;
        let video = self.video.as_ref()?;

        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        for i in 0..(video.width * video.height) as usize {
            let pixel_r = unsafe { *frame_data_ptr.add(i * 3 + 2) };
            let pixel_g = unsafe { *frame_data_ptr.add(i * 3 + 1) };
            let pixel_b = unsafe { *frame_data_ptr.add(i * 3) };
            frame_buffer.push((pixel_r, pixel_g, pixel_b));
        }
        for y in 0..(video.height as usize / 2) {
            for x in 0..video.width as usize {
                frame_buffer.swap(
                    y * video.width as usize + x,
                    (video.height as usize - 1 - y) * video.width as usize + x,
                );
            }
        }

        Some(frame_buffer)
    }

    pub fn get_video_frames_iter(&self) -> VideoFramesIterator {
        VideoFramesIterator::new(self)
    }

    pub fn get_audio_samples(&self, start: i32, length: i32) -> Option<(Vec<f32>, u32)> {
        let audio_ptr = unsafe {
            (self.internal as *mut aviutl2_sys::output2::OUTPUT_INFO)
                .as_mut()
                .and_then(|oip| oip.func_get_audio)
        }?;
        let mut readed = 0;
        let audio_data_ptr = audio_ptr(start, length, &mut readed, 3) as *mut f32;

        let audio = self.audio.as_ref()?;
        let samples = unsafe {
            std::slice::from_raw_parts(
                audio_data_ptr,
                ((readed as u32) * audio.num_channels) as usize,
            )
            .to_vec()
        };

        Some((samples, audio.num_channels))
    }

    pub fn get_mono_audio_samples(&self, start: i32, length: i32) -> Option<Vec<f32>> {
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

    pub fn get_mono_audio_samples_iter(&self, length: i32) -> MonoAudioSamplesIterator {
        MonoAudioSamplesIterator::new(self, 0, length)
    }

    pub fn get_stereo_audio_samples(&self, start: i32, length: i32) -> Option<Vec<(f32, f32)>> {
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

    pub fn get_stereo_audio_samples_iter(&self, length: i32) -> StereoAudioSamplesIterator {
        StereoAudioSamplesIterator::new(self, 0, length)
    }

    pub fn is_aborted(&self) -> bool {
        let is_abort_func = unsafe {
            (self.internal as *mut aviutl2_sys::output2::OUTPUT_INFO)
                .as_mut()
                .and_then(|oip| oip.func_is_abort)
        };
        is_abort_func.map_or(true, |f| f())
    }

    pub fn update_display(&self, current_frame: i32, total_frames: i32) {
        if let Some(func) = unsafe {
            (self.internal as *mut aviutl2_sys::output2::OUTPUT_INFO)
                .as_mut()
                .and_then(|oip| oip.func_rest_time_disp)
        } {
            func(current_frame, total_frames);
        }
    }

    pub fn set_buffer_size(&self, video_size: i32, audio_size: i32) {
        if let Some(func) = unsafe {
            (self.internal as *mut aviutl2_sys::output2::OUTPUT_INFO)
                .as_mut()
                .and_then(|oip| oip.func_set_buffer_size)
        } {
            func(video_size, audio_size);
        }
    }
}

pub struct VideoFramesIterator<'a> {
    output_info: &'a OutputInfo,
    current_frame: i32,
    total_frames: i32,
    last_updated_time: std::time::Instant,
}

impl<'a> VideoFramesIterator<'a> {
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
        }
    }
}

impl<'a> Iterator for VideoFramesIterator<'a> {
    type Item = (i32, VideoFrame);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame >= self.total_frames {
            return None;
        }

        if self.output_info.is_aborted() {
            return None;
        }

        let frame = self.output_info.get_video_frame(self.current_frame);
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
        [MonoAudioSamplesIterator] [get_mono_audio_samples] [f32];
        [StereoAudioSamplesIterator] [get_stereo_audio_samples] [(f32, f32)];
    ]

    pub struct Name<'a> {
        output_info: &'a OutputInfo,
        start: i32,
        length: i32,
        readed: i32,
    }

    impl<'a> Name<'a> {
        pub fn new(output_info: &'a OutputInfo, start: i32, length: i32) -> Self {
            Self {
                output_info,
                start,
                length,
                readed: 0,
            }
        }
    }

    impl<'a> Iterator for Name<'a> {
        type Item = (usize, Vec<IterType>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.readed >= self.length {
                return None;
            }
            if self.output_info.is_aborted() {
                return None;
            }

            let remaining_length = self.length - self.readed;
            let samples = self.output_info.method(self.start + self.readed, remaining_length);
            if let Some(samples) = samples {
                let start_frame = self.start + self.readed;
                self.readed += samples.len() as i32;
                Some((start_frame as usize, samples))
            } else {
                None
            }
        }
    }
}

pub fn create_table<T: OutputPlugin>(
    plugin: &T,
    func_output: extern "C" fn(*mut aviutl2_sys::output2::OUTPUT_INFO) -> bool,
    func_config: Option<
        extern "C" fn(aviutl2_sys::output2::HWND, aviutl2_sys::output2::HINSTANCE) -> bool,
    >,
    func_get_config_text: Option<extern "C" fn() -> LPCWSTR>,
) -> aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
    free_leaked_memory();
    let plugin_info = plugin.plugin_info();
    let name = leak_large_string(&plugin_info.name);
    let filefilter = format_file_filters(&plugin_info.file_filters);
    let filefilter = leak_large_string(&filefilter);
    let information = leak_large_string(&plugin_info.information);

    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE {
        flag: plugin_info.input_type.to_bits(),
        name,
        filefilter,
        information,
        func_output: Some(func_output),
        func_config,
        func_get_config_text,
    }
}

pub fn func_output<T: OutputPlugin>(
    plugin: &T,
    oip: *mut aviutl2_sys::output2::OUTPUT_INFO,
) -> bool {
    let oip = unsafe { &mut *oip };
    let output_info = OutputInfo::from_raw(oip);
    result_to_bool_with_dialog(plugin.output(output_info))
}

pub fn func_config<T: OutputPlugin>(
    plugin: &T,
    hwnd: aviutl2_sys::output2::HWND,
    dll_hinst: aviutl2_sys::output2::HINSTANCE,
) -> bool {
    result_to_bool_with_dialog(plugin.config(hwnd, dll_hinst))
}

pub fn func_get_config_text<T: OutputPlugin>(plugin: &T) -> *mut u16 {
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
        mod __au2_register_plugin {
            use super::*;

            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new($struct::new);

            #[unsafe(no_mangle)]
            extern "C" fn GetOutputPluginTable() -> *mut aviutl2::sys::output2::OUTPUT_PLUGIN_TABLE
            {
                let table = $crate::output::__bridge::create_table::<$struct>(
                    &*PLUGIN,
                    func_output,
                    Some(func_config),
                    Some(func_get_config_text),
                );
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
