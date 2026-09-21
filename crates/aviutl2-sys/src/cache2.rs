#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::ffi::c_void;

use crate::{
    common::LPCWSTR,
    filter2::{INPUT_PIXEL_FORMAT, PIXEL_RGBA},
};

/// キャッシュデータ参照の基底クラス
/// クラスオブジェクトが生存している間はキャッシュ参照が有効となるように制御されます
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
    /// ※画像データはPIXEL_RGBA
    pub buffer: *mut PIXEL_RGBA,
    /// 画像キャッシュの画像サイズ
    pub width: i32,
    /// 画像キャッシュの画像サイズ
    pub height: i32,
}

/// 音声キャッシュデータ構造体
#[repr(C)]
pub struct CACHE_AUDIO {
    pub reference: CACHE_REFERENCE,
    /// 音声キャッシュデータ(左チャンネル)へのポインタ (取得失敗時はnullptr)
    /// ※音声データはPCM(float)32bit
    pub buffer0: *mut f32,
    /// 音声キャッシュデータ(右チャンネル)へのポインタ (取得失敗時はnullptr)
    /// ※音声データはPCM(float)32bit
    pub buffer1: *mut f32,
    /// 音声キャッシュのサンプル数
    pub sample_num: i32,
    /// 音声キャッシュのチャンネル数 ( 1 = モノラル / 2 = ステレオ )
    /// チャンネル数が1の場合は buffer0 のみ利用出来ます
    pub channel_num: i32,
}

/// メディアファイルの画像キャッシュデータ構造体
#[repr(C)]
pub struct CACHE_FILE_IMAGE {
    pub reference: CACHE_REFERENCE,
    /// 画像キャッシュデータへのポインタ (取得失敗時はnullptr)
    /// ※画像データはINPUT_PIXEL_FORMATのいずれかになります
    pub buffer: *const c_void,
    /// 画像キャッシュの画像サイズ
    pub width: i32,
    /// 画像キャッシュの画像サイズ
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
    /// 解像度
    pub height: i32,
    /// フレームレート
    pub rate: i32,
    /// フレームレート
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
/// アプリケーションの共用のキャッシュ領域に各種キャッシュデータを作成することが出来ます
/// ※スクリプトのキャッシュバッファ(cache:xxxx)とは異なりメインメモリに確保されます
#[repr(C)]
pub struct CACHE_HANDLE {
    /// 画像キャッシュデータを取得する
    /// identifier : キャッシュ識別のポインタ ※任意の静的なポインタを指定する(CACHE_HANDLEやFILTER_PLUGIN_TABLE等)
    /// name : キャッシュ識別の名前 ※作成した画像キャッシュの識別名
    /// 戻り値 : 画像キャッシュデータ
    /// 取得出来ない場合は返却オブジェクトがfalseとなる
    pub get_image_cache:
        unsafe extern "C" fn(identifier: *mut c_void, name: LPCWSTR) -> CACHE_IMAGE,

    /// 画像キャッシュデータを作成する
    /// identifier : キャッシュ識別のポインタ ※任意の静的なポインタを指定する(CACHE_HANDLEやFILTER_PLUGIN_TABLE等)
    /// name : キャッシュ識別の名前 ※任意の識別名を付けることが出来る
    /// width,height : 作成するキャッシュの画像サイズ
    /// 戻り値 : 画像キャッシュデータ
    /// 返却されたキャッシュに画像データを書き込むことが出来る
    pub create_image_cache: unsafe extern "C" fn(
        identifier: *mut c_void,
        name: LPCWSTR,
        width: i32,
        height: i32,
    ) -> CACHE_IMAGE,

    /// 音声キャッシュデータを取得する
    /// identifier : キャッシュ識別のポインタ ※任意の静的なポインタを指定する(CACHE_HANDLEやFILTER_PLUGIN_TABLE等)
    /// name : キャッシュ識別の名前 ※作成した音声キャッシュの識別名
    /// 戻り値 : 音声キャッシュデータ
    /// 取得出来ない場合は返却オブジェクトがfalseとなる
    pub get_audio_cache:
        unsafe extern "C" fn(identifier: *mut c_void, name: LPCWSTR) -> CACHE_AUDIO,

    /// 音声キャッシュデータを作成する
    /// identifier : キャッシュ識別のポインタ ※任意の静的なポインタを指定する(CACHE_HANDLEやFILTER_PLUGIN_TABLE等)
    /// name : キャッシュ識別の名前 ※任意の識別名を付けることが出来る
    /// sample_num : 作成する音声キャッシュのサンプル数
    /// channel_num : 作成する音声キャッシュのチャンネル数 ( 1 = モノラル / 2 = ステレオ )
    /// 戻り値 : 音声キャッシュデータ
    /// 返却されたキャッシュに音声データを書き込むことが出来る
    pub create_audio_cache: unsafe extern "C" fn(
        identifier: *mut c_void,
        name: LPCWSTR,
        sample_num: i32,
        channel_num: i32,
    ) -> CACHE_AUDIO,

    /// 新しい関数に差し替えるので廃止します
    pub deprecated_get_image_file_cache: unsafe extern "C" fn(file: LPCWSTR) -> CACHE_IMAGE,

    /// メディアファイルのビデオ情報を取得する
    /// file : メディアファイルのパス
    /// info : ビデオ情報の格納先へのポインタ
    /// info_size : ビデオ情報の格納先のサイズ ※VIDEO_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue
    pub get_video_file_info:
        unsafe extern "C" fn(file: LPCWSTR, info: *mut VIDEO_INFO, info_size: i32) -> bool,

    /// メディアファイルのオーディオ情報を取得する
    /// file : メディアファイルのパス
    /// info : オーディオ情報の格納先へのポインタ
    /// info_size : オーディオ情報の格納先のサイズ ※AUDIO_INFOと異なる場合はサイズ分のみ取得されます
    /// 戻り値 : 取得出来た場合はtrue
    pub get_audio_file_info:
        unsafe extern "C" fn(file: LPCWSTR, info: *mut AUDIO_INFO, info_size: i32) -> bool,

    /// 画像ファイルから画像データをキャッシュ経由で取得する
    /// file : 画像ファイルのパス
    /// 戻り値 : 画像キャッシュデータ
    /// 取得出来ない場合は返却オブジェクトがfalseとなる
    pub get_image_file_cache: unsafe extern "C" fn(file: LPCWSTR) -> CACHE_FILE_IMAGE,

    /// メディアファイルから画像データをキャッシュ経由で取得する
    /// file : メディアファイルのパス
    /// track : トラック番号
    /// frame : 取得するフレーム番号
    /// 戻り値 : 画像キャッシュデータ
    /// 取得出来ない場合は返却オブジェクトがfalseとなる
    pub get_video_file_cache:
        unsafe extern "C" fn(file: LPCWSTR, track: i32, frame: i32) -> CACHE_FILE_IMAGE,

    /// メディアファイルから画像データをキャッシュ経由で取得する
    /// file : メディアファイルのパス
    /// track : ビデオトラック番号
    /// time : 取得するフレームの時間
    /// 戻り値 : 画像キャッシュデータ
    /// 取得出来ない場合は返却オブジェクトがfalseとなる
    pub get_video_file_cache_by_time:
        unsafe extern "C" fn(file: LPCWSTR, track: i32, time: f64) -> CACHE_FILE_IMAGE,

    /// メディアファイルから音声データをキャッシュ経由で取得する
    /// ※音声データはPCM(float)32bit2ch
    /// file : メディアファイルのパス
    /// track : オーディオトラック番号
    /// sample_index : 取得するサンプル位置
    /// sample_num : 取得するサンプル数
    /// buffer0 : サンプル(左チャンネル)取得先のバッファへのポインタ
    /// buffer1 : サンプル(右チャンネル)取得先のバッファへのポインタ
    /// 戻り値 : 実際に取得したサンプル数
    pub get_audio_file_data: unsafe extern "C" fn(
        file: LPCWSTR,
        track: i32,
        sample_index: i64,
        sample_num: i32,
        buffer0: *mut f32,
        buffer1: *mut f32,
    ) -> i32,

    /// 画像キャッシュデータをクリアする
    /// identifier : キャッシュ識別のポインタ ※任意の静的なポインタを指定する(CACHE_HANDLEやFILTER_PLUGIN_TABLE等)
    /// name : キャッシュ識別の名前 ※作成した画像キャッシュの識別名
    pub clear_image_cache: unsafe extern "C" fn(identifier: *mut c_void, name: LPCWSTR),

    /// 音声キャッシュデータをクリアする
    /// identifier : キャッシュ識別のポインタ ※任意の静的なポインタを指定する(CACHE_HANDLEやFILTER_PLUGIN_TABLE等)
    /// name : キャッシュ識別の名前 ※作成した音声キャッシュの識別名
    pub clear_audio_cache: unsafe extern "C" fn(identifier: *mut c_void, name: LPCWSTR),
}
