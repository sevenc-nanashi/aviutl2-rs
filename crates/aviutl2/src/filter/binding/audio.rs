use super::{FilterProcError, FilterProcResult, ObjectInfo, SceneInfo};

/// オブジェクトの音声パラメータ構造体。
#[derive(Debug, Clone, Copy)]
pub struct ObjectAudioParam {
    /// 左チャンネルの音量倍率。（1.0=等倍）
    pub vol_l: f32,
    /// 右チャンネルの音量倍率。（1.0=等倍）
    pub vol_r: f32,
}
impl From<&aviutl2_sys::filter2::OBJECT_AUDIO_PARAM> for ObjectAudioParam {
    fn from(value: &aviutl2_sys::filter2::OBJECT_AUDIO_PARAM) -> Self {
        Self {
            vol_l: value.vol_l,
            vol_r: value.vol_r,
        }
    }
}
impl From<aviutl2_sys::filter2::OBJECT_AUDIO_PARAM> for ObjectAudioParam {
    fn from(value: aviutl2_sys::filter2::OBJECT_AUDIO_PARAM) -> Self {
        Self {
            vol_l: value.vol_l,
            vol_r: value.vol_r,
        }
    }
}

/// 音声フィルタのオブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct AudioObjectInfo {
    /// オブジェクトの現在の音声サンプル位置。
    pub sample_index: u64,
    /// オブジェクトの総サンプル数。
    pub sample_total: u64,
    /// オブジェクトの現在の音声サンプル数。
    pub sample_num: u32,
    /// オブジェクトの現在の音声チャンネル数。
    /// 通常2になります。
    pub channel_num: u32,
}

/// 音声フィルタ処理のための構造体。
#[derive(Debug)]
pub struct FilterProcAudio {
    /// シーン情報。
    pub scene: SceneInfo,
    /// オブジェクト情報。
    pub object: ObjectInfo,
    /// 音声フィルタ特有のオブジェクト情報。
    pub audio_object: AudioObjectInfo,

    /// オブジェクトの音声パラメータ。
    ///
    /// # Note
    ///
    /// このパラメータは音声出力項目のパラメータからの相対設定になります。
    pub param: ObjectAudioParam,

    pub(crate) read: crate::generic::ReadSection,
    pub(crate) inner: *const aviutl2_sys::filter2::FILTER_PROC_AUDIO,
}

unsafe impl Send for FilterProcAudio {}
unsafe impl Sync for FilterProcAudio {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    Left,
    Right,
    Any(i32),
}
impl From<i32> for AudioChannel {
    fn from(value: i32) -> Self {
        match value {
            0 => AudioChannel::Left,
            1 => AudioChannel::Right,
            v => AudioChannel::Any(v),
        }
    }
}
impl From<AudioChannel> for i32 {
    fn from(value: AudioChannel) -> Self {
        match value {
            AudioChannel::Left => 0,
            AudioChannel::Right => 1,
            AudioChannel::Any(v) => v,
        }
    }
}

impl FilterProcAudio {
    /// 現在の音声のデータを取得する。
    /// `channel` は 0 が左チャンネル、1 が右チャンネルです。
    ///
    /// # Panics
    ///
    /// `buffer` の長さが `sample_num` と一致しない場合、パニックします。
    pub fn get_sample_data(&mut self, channel: AudioChannel, buffer: &mut [f32]) -> usize {
        let sample_num = self.audio_object.sample_num as usize;
        assert_eq!(
            buffer.len(),
            sample_num,
            "buffer length does not match sample_num"
        );
        let inner = unsafe { &*self.inner };
        unsafe { (inner.get_sample_data)(buffer.as_mut_ptr(), channel.into()) };
        sample_num
    }

    /// 現在の音声のデータを設定する。
    /// `channel` は 0 が左チャンネル、1 が右チャンネルです。
    ///
    /// # Panics
    ///
    /// `data` の長さが `sample_num` と一致しない場合、パニックします。
    pub fn set_sample_data(&mut self, channel: AudioChannel, data: &[f32]) {
        let sample_num = self.audio_object.sample_num as usize;
        assert_eq!(
            data.len(),
            sample_num,
            "data length does not match sample_num"
        );
        let inner = unsafe { &*self.inner };
        unsafe { (inner.set_sample_data)(data.as_ptr(), channel.into()) };
    }

    /// 読み取り専用の編集セクション。
    pub fn read_section(&mut self) -> &crate::generic::ReadSection {
        &self.read
    }

    /// 指定オブジェクトの音声出力項目のパラメーターを取得する。
    pub fn get_output_audio_param(
        &self,
        object: crate::generic::ObjectHandle,
        offset: f64,
    ) -> FilterProcResult<ObjectAudioParam> {
        let inner = unsafe { &*self.inner };
        let mut param = aviutl2_sys::filter2::OBJECT_AUDIO_PARAM {
            vol_l: 0.0,
            vol_r: 0.0,
        };
        let ok = unsafe {
            (inner.get_output_audio_param)(
                object.internal,
                offset,
                &mut param,
                std::mem::size_of::<aviutl2_sys::filter2::OBJECT_AUDIO_PARAM>() as i32,
            )
        };
        if ok {
            Ok(param.into())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 指定のレイヤー位置にある音声オブジェクトを取得する。
    pub fn get_audio_object(
        &self,
        layer: u32,
        offset: f64,
    ) -> Option<crate::generic::ObjectHandle> {
        let handle = unsafe { ((*self.inner).get_audio_object)(layer as _, offset) };
        if handle.is_null() {
            None
        } else {
            Some(crate::generic::ObjectHandle { internal: handle })
        }
    }

    pub(crate) fn apply_param(&mut self) {
        let inner = unsafe { &mut *(*self.inner).param };
        inner.vol_l = self.param.vol_l;
        inner.vol_r = self.param.vol_r;
    }
}
