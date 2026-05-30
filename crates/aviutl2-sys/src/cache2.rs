#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::ffi::c_void;

use crate::{
    common::LPCWSTR,
    filter2::{INPUT_PIXEL_FORMAT, PIXEL_RGBA},
};

/// キャッシュデータ参照の基底クラス
///
/// クラスオブジェクトが生存している間はキャッシュ参照が有効となるように制御されます。
#[repr(C)]
pub struct CACHE_REFERENCE {
    pub func_release: Option<unsafe extern "C" fn(instance: *mut c_void)>,
    pub cache_instance: *mut c_void,
}

impl Drop for CACHE_REFERENCE {
    fn drop(&mut self) {
        if let Some(func_release) = self.func_release
            && !self.cache_instance.is_null()
        {
            unsafe {
                func_release(self.cache_instance);
            }
        }
    }
}

/// 画像キャッシュデータ構造体
#[repr(C)]
pub struct CACHE_IMAGE {
    pub reference: CACHE_REFERENCE,
    /// 画像キャッシュデータへのポインタ (取得失敗時はnullptr)
    pub buffer: *mut PIXEL_RGBA,
    /// 画像キャッシュの画像サイズ
    pub width: i32,
    pub height: i32,
}

/// 音声キャッシュデータ構造体
#[repr(C)]
pub struct CACHE_AUDIO {
    pub reference: CACHE_REFERENCE,
    /// 音声キャッシュデータ(左チャンネル)へのポインタ (取得失敗時はnullptr)
    pub buffer0: *mut f32,
    /// 音声キャッシュデータ(右チャンネル)へのポインタ (取得失敗時はnullptr)
    pub buffer1: *mut f32,
    /// 音声キャッシュのサンプル数
    pub sample_num: i32,
    /// 音声キャッシュのチャンネル数 ( 1 = モノラル / 2 = ステレオ )
    pub channel_num: i32,
}

/// メディアファイルの画像キャッシュデータ構造体
#[repr(C)]
pub struct CACHE_FILE_IMAGE {
    pub reference: CACHE_REFERENCE,
    /// 画像キャッシュデータへのポインタ (取得失敗時はnullptr)
    pub buffer: *const c_void,
    /// 画像キャッシュの画像サイズ
    pub width: i32,
    pub height: i32,
    /// 画像キャッシュデータの横1ラインのバイト数
    pub pitch: i32,
    /// 画像キャッシュのピクセルフォーマット
    pub format: INPUT_PIXEL_FORMAT,
}

/// ビデオ情報構造体
#[repr(C)]
pub struct VIDEO_INFO {
    /// 総時間
    pub total_time: f64,
    /// 総フレーム数
    pub frame_num: i32,
    /// トラック数
    pub track_num: i32,
    /// 解像度
    pub width: i32,
    pub height: i32,
    /// フレームレート
    pub rate: i32,
    pub scale: i32,
}

/// オーディオ情報構造体
#[repr(C)]
pub struct AUDIO_INFO {
    /// 総時間
    pub total_time: f64,
    /// 総サンプル数
    pub sample_num: i64,
    /// トラック数
    pub track_num: i32,
    /// サンプリングレート
    pub rate: i32,
    /// チャンネル数
    pub channel: i32,
}

/// キャッシュハンドル
#[repr(C)]
pub struct CACHE_HANDLE {
    /// 画像キャッシュデータを取得する
    pub get_image_cache:
        unsafe extern "C" fn(identifier: *mut c_void, name: LPCWSTR) -> CACHE_IMAGE,

    /// 画像キャッシュデータを作成する
    pub create_image_cache: unsafe extern "C" fn(
        identifier: *mut c_void,
        name: LPCWSTR,
        width: i32,
        height: i32,
    ) -> CACHE_IMAGE,

    /// 音声キャッシュデータを取得する
    pub get_audio_cache:
        unsafe extern "C" fn(identifier: *mut c_void, name: LPCWSTR) -> CACHE_AUDIO,

    /// 音声キャッシュデータを作成する
    pub create_audio_cache: unsafe extern "C" fn(
        identifier: *mut c_void,
        name: LPCWSTR,
        sample_num: i32,
        channel_num: i32,
    ) -> CACHE_AUDIO,

    /// 新しい関数に差し替えるので廃止します
    pub deprecated_get_image_file_cache: unsafe extern "C" fn(file: LPCWSTR) -> CACHE_IMAGE,

    /// メディアファイルのビデオ情報を取得する
    pub get_video_file_info:
        unsafe extern "C" fn(file: LPCWSTR, info: *mut VIDEO_INFO, info_size: i32) -> bool,

    /// メディアファイルのオーディオ情報を取得する
    pub get_audio_file_info:
        unsafe extern "C" fn(file: LPCWSTR, info: *mut AUDIO_INFO, info_size: i32) -> bool,

    /// 画像ファイルから画像データをキャッシュ経由で取得する
    pub get_image_file_cache: unsafe extern "C" fn(file: LPCWSTR) -> CACHE_FILE_IMAGE,

    /// メディアファイルから画像データをキャッシュ経由で取得する
    pub get_video_file_cache:
        unsafe extern "C" fn(file: LPCWSTR, track: i32, frame: i32) -> CACHE_FILE_IMAGE,

    /// メディアファイルから画像データをキャッシュ経由で取得する
    pub get_video_file_cache_by_time:
        unsafe extern "C" fn(file: LPCWSTR, track: i32, time: f64) -> CACHE_FILE_IMAGE,

    /// メディアファイルから音声データをキャッシュ経由で取得する
    pub get_audio_file_data: unsafe extern "C" fn(
        file: LPCWSTR,
        track: i32,
        sample_index: i64,
        sample_num: i32,
        buffer0: *mut f32,
        buffer1: *mut f32,
    ) -> i32,
}
