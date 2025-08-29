use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::common::{AnyResult, FileFilter, Rational32, Win32WindowHandle, load_large_string};
use crate::output::FromRawVideoFrame;
use aviutl2_sys::output2::OUTPUT_INFO;

/// 出力プラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct OutputPluginTable {
    /// プラグインの名前。
    pub name: String,
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,
    /// 出力の種類（動画、音声、両方）。
    pub output_type: OutputType,

    /// 出力ダイアログで使われるファイルフィルタ。
    pub file_filters: Vec<FileFilter>,

    /// 設定ダイアログがあるかどうか。
    pub can_config: bool,
}

/// 出力の種類を表す列挙型。
#[derive(Debug, Clone)]
pub enum OutputType {
    /// 動画のみの出力。
    Video,
    /// 音声のみの出力。
    Audio,
    /// 動画と音声の両方を出力。
    Both,
}

impl OutputType {
    pub(crate) fn to_bits(&self) -> i32 {
        match self {
            OutputType::Video => 1,
            OutputType::Audio => 2,
            OutputType::Both => 3,
        }
    }
}

/// 出力情報を表す構造体。
#[derive(Debug, Clone)]
pub struct OutputInfo {
    /// 動画出力情報。動画出力がない場合は`None`。
    pub video: Option<VideoOutputInfo>,
    /// 音声出力情報。音声出力がない場合は`None`。
    pub audio: Option<AudioOutputInfo>,
    /// 出力先のファイルパス。
    pub path: std::path::PathBuf,

    pub(crate) internal: *mut OUTPUT_INFO,
    pub(crate) current_frame: Arc<AtomicUsize>,
}

unsafe impl Send for OutputInfo {}
unsafe impl Sync for OutputInfo {}

/// 動画の出力情報を表す構造体。
#[derive(Debug, Clone)]
pub struct VideoOutputInfo {
    /// 動画の幅（ピクセル単位）。
    pub width: u32,
    /// 動画の高さ（ピクセル単位）。
    pub height: u32,
    /// 動画のフレームレート（分数形式）。
    pub fps: Rational32,
    /// 動画のフレーム数。
    pub num_frames: u32,
}

/// 音声の出力情報を表す構造体。
#[derive(Debug, Clone)]
pub struct AudioOutputInfo {
    /// 音声のサンプルレート（Hz単位）。
    pub sample_rate: u32,
    /// 音声のサンプル数。
    pub num_samples: u32,
    /// 音声のチャンネル数。
    pub num_channels: u32,
}

/// 出力プラグインのトレイト。
/// このトレイトを実装し、[`crate::register_output_plugin!`] マクロを使用してプラグインを登録します。
pub trait OutputPlugin: Send + Sync {
    /// プラグインを初期化する。
    fn new() -> Self;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> OutputPluginTable;

    /// 出力を開始する。
    fn output(&self, info: OutputInfo) -> AnyResult<()>;

    /// 出力設定のダイアログを表示する。
    fn config(&self, _hwnd: Win32WindowHandle) -> AnyResult<()> {
        Ok(())
    }

    /// 出力設定のテキスト情報を返す。
    /// 出力ダイアログの下の設定ボタンの隣に表示されます。
    fn config_text(&self) -> AnyResult<String> {
        Ok(String::new())
    }
}

/// 音声サンプルを表すトレイト。
/// aviutl2-rsでは、このトレイトを実装した型で音声サンプルのフォーマットを指定します。
pub trait FromRawAudioSamples: Sized + Send + Sync + Copy {
    /// 音声サンプルのフォーマットを表す定数。
    const FORMAT: u32;

    /// 音声サンプルの情報を生のポインタから取得する。
    ///
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
            current_frame: Arc::new(AtomicUsize::new(usize::MAX)),
        }
    }

    /// 動画のフレームを取得する。
    pub fn get_video_frame<F: FromRawVideoFrame>(&self, frame: i32) -> Option<F> {
        if let Some(video) = &self.video {
            if F::check(video).is_err() {
                return None;
            }
            if frame < 0 || frame >= video.num_frames as i32 {
                return None;
            }
            unsafe { self.get_video_frame_unchecked::<F>(frame) }
        } else {
            None
        }
    }

    /// 動画のフレームを取得する。
    /// [`Self::get_video_frame`]と違い、[`FromRawVideoFrame::check`]や境界のチェックを行いません。
    ///
    /// # Safety
    /// 以下は未定義動作です：
    /// - [`FromRawVideoFrame::check`]がfalseの場合
    /// - `frame`が動画のフレーム数の範囲外の場合
    pub unsafe fn get_video_frame_unchecked<F: FromRawVideoFrame>(&self, frame: i32) -> Option<F> {
        let frame_ptr = unsafe { self.internal.as_mut().and_then(|oip| oip.func_get_video) }?;
        let frame_data_ptr = frame_ptr(frame, F::FORMAT) as *mut u8;
        let video = self.video.as_ref()?;
        self.current_frame.store(frame as usize, Ordering::SeqCst);
        let frame = unsafe { F::from_raw(video, frame_data_ptr, self.current_frame.clone(), frame as usize) };
        Some(frame)
    }

    /// 動画のフレームをイテレータとして取得する。
    pub fn get_video_frames_iter<F: FromRawVideoFrame>(&self) -> VideoFramesIterator<'_, F> {
        VideoFramesIterator::new(self)
    }

    /// 指定した区間の音声サンプルとチャンネル数を取得する。
    pub fn get_audio_samples<F: FromRawAudioSamples>(
        &self,
        start: i32,
        length: i32,
    ) -> Option<(Vec<F>, u32)> {
        let audio = self.audio.as_ref()?;
        let audio_ptr = unsafe { self.internal.as_mut().and_then(|oip| oip.func_get_audio) }?;
        let mut readed = 0;
        let audio_data_ptr = audio_ptr(start, length, &mut readed, F::FORMAT) as *mut u8;

        let samples = unsafe { F::from_raw(length, audio.num_channels, audio_data_ptr) };

        Some((samples, audio.num_channels))
    }

    /// 指定した区間の音声サンプルをモノラル形式で取得する。
    /// `num_channels`が1の場合はそのまま、2の場合は左チャンネルのサンプルを返します。
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

    /// モノラルの音声サンプルをイテレータとして取得する。
    ///
    /// # Arguments
    /// `length` - 一回のイテレーションで取得するサンプル数。
    pub fn get_mono_audio_samples_iter<F: FromRawAudioSamples>(
        &self,
        length: i32,
    ) -> MonoAudioSamplesIterator<F> {
        MonoAudioSamplesIterator::new(self, length)
    }

    /// 指定した区間の音声サンプルをステレオ形式で取得する。
    /// `num_channels`が2の場合はそのまま、1の場合はチャンネルを複製してステレオ形式に変換します。
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

    /// ステレオの音声サンプルをイテレータとして取得する。
    ///
    /// # Arguments
    /// `length` - 一回のイテレーションで取得するサンプル数。
    pub fn get_stereo_audio_samples_iter<F: FromRawAudioSamples>(
        &self,
        length: i32,
    ) -> StereoAudioSamplesIterator<F> {
        StereoAudioSamplesIterator::new(self, length)
    }

    /// 出力が中断されたかどうかを確認する。
    pub fn is_aborted(&self) -> bool {
        let is_abort_func = unsafe { self.internal.as_mut().and_then(|oip| oip.func_is_abort) };
        is_abort_func.is_none_or(|f| f())
    }

    /// 出力の進行状況を更新する。
    pub fn update_display(&self, current_frame: i32, total_frames: i32) {
        if let Some(func) = unsafe {
            self.internal
                .as_mut()
                .and_then(|oip| oip.func_rest_time_disp)
        } {
            func(current_frame, total_frames);
        }
    }

    /// データ取得のバッファ数（フレーム数）を設定する。
    /// バッファ数の半分のデータを先読みリクエストするようになります。
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

/// 動画フレームのイテレータ。
///
/// # See Also
/// [`OutputInfo::get_video_frames_iter`]
#[derive(Debug, Clone)]
pub struct VideoFramesIterator<'a, F: FromRawVideoFrame> {
    output_info: &'a OutputInfo,
    current_frame: i32,
    total_frames: i32,
    last_updated_time: std::time::Instant,
    check_result: bool,
    _marker: std::marker::PhantomData<F>,
}

impl<'a, F: FromRawVideoFrame> VideoFramesIterator<'a, F> {
    pub(crate) fn new(output_info: &'a OutputInfo) -> Self {
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
        Name                         method                     IterType Doc                                    Also;
        [MonoAudioSamplesIterator]   [get_mono_audio_samples]   [F]      ["モノラル音声サンプルのイテレータ。"] ["[`OutputInfo::get_mono_audio_samples_iter`]"];
        [StereoAudioSamplesIterator] [get_stereo_audio_samples] [(F, F)] ["ステレオ音声サンプルのイテレータ。"] ["[`OutputInfo::get_stereo_audio_samples_iter`]"];
    ]

    #[doc = Doc]
    ///
    /// # See Also
    #[doc = Also]
    #[derive(Debug, Clone)]
    pub struct Name<'a, F: FromRawAudioSamples> {
        output_info: &'a OutputInfo,
        length: i32,
        total_length: i32,
        readed: i32,
        _marker: std::marker::PhantomData<F>,
    }

    impl<'a, F: FromRawAudioSamples> Name<'a, F> {
        pub(crate) fn new(output_info: &'a OutputInfo, length: i32) -> Self {
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
