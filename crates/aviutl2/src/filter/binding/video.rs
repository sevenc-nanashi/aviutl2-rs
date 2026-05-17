use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use super::{FilterProcError, FilterProcResult, ObjectInfo, SceneInfo};

/// 画像フィルタのオブジェクト情報。
#[derive(Debug, Clone, Copy)]
pub struct VideoObjectInfo {
    /// オブジェクトの現在の画像サイズの幅。
    pub width: u32,
    /// オブジェクトの現在の画像サイズの高さ。
    pub height: u32,

    /// 複数オブジェクト時の現在の対象番号
    pub index: u32,
    /// 複数オブジェクト時の対象数（None：不定、Some(1)：単体）
    pub num: Option<u32>,
}

/// オブジェクトの画像パラメータ構造体。
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct ObjectImageParam {
    /// 基準座標X。
    pub x: f32,
    /// 基準座標Y。
    pub y: f32,
    /// 基準座標Z。
    pub z: f32,
    /// 回転角度X。（360.0で1回転）
    pub rx: f32,
    /// 回転角度Y。（360.0で1回転）
    pub ry: f32,
    /// 回転角度Z。（360.0で1回転）
    pub rz: f32,
    /// 拡大率X。（1.0で等倍）
    pub sx: f32,
    /// 拡大率Y。（1.0で等倍）
    pub sy: f32,
    /// 拡大率Z。（1.0で等倍）
    pub sz: f32,
    /// 中心座標X。（基準座標からの相対）
    pub cx: f32,
    /// 中心座標Y。（基準座標からの相対）
    pub cy: f32,
    /// 中心座標Z。（基準座標からの相対）
    pub cz: f32,
    /// 不透明度。（0.0〜1.0/0.0=透明/1.0=不透明）
    pub alpha: f32,
}

impl From<aviutl2_sys::filter2::OBJECT_IMAGE_PARAM> for ObjectImageParam {
    fn from(value: aviutl2_sys::filter2::OBJECT_IMAGE_PARAM) -> Self {
        (&value).into()
    }
}

impl From<&aviutl2_sys::filter2::OBJECT_IMAGE_PARAM> for ObjectImageParam {
    fn from(value: &aviutl2_sys::filter2::OBJECT_IMAGE_PARAM) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            rx: value.rx,
            ry: value.ry,
            rz: value.rz,
            sx: value.sx,
            sy: value.sy,
            sz: value.sz,
            cx: value.cx,
            cy: value.cy,
            cz: value.cz,
            alpha: value.alpha,
        }
    }
}

/// RGBAのピクセル。
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, IntoBytes, FromBytes, Immutable, KnownLayout,
)]
pub struct RgbaPixel {
    /// 赤。
    pub r: u8,
    /// 緑。
    pub g: u8,
    /// 青。
    pub b: u8,
    /// アルファ。
    pub a: u8,
}

/// 画像フィルタ処理のための構造体。
#[derive(Debug)]
pub struct FilterProcVideo {
    /// シーン情報。
    pub scene: SceneInfo,
    /// オブジェクト情報。
    pub object: ObjectInfo,
    /// 画像フィルタ特有のオブジェクト情報。
    pub video_object: VideoObjectInfo,

    /// オブジェクトの画像パラメータ。
    /// [`crate::filter::FilterPlugin::proc_video`] の終了時に反映されます。
    pub param: ObjectImageParam,

    pub(crate) read: crate::generic::ReadSection,
    pub(crate) inner: *const aviutl2_sys::filter2::FILTER_PROC_VIDEO,
}
unsafe impl Send for FilterProcVideo {}
unsafe impl Sync for FilterProcVideo {}

/// 描画時の画像リソース。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawImageResource {
    /// 現在の画像データ。
    Object,
    /// 仮想バッファ。
    TempBuffer,
    /// キャッシュバッファ。
    CacheBuffer(String),
    /// 画像ファイル。
    ImageFile(std::path::PathBuf),
}
impl DrawImageResource {
    fn to_cw_string(
        &self,
    ) -> Result<Option<crate::common::CWString>, crate::common::NullByteError> {
        Ok(match self {
            DrawImageResource::Object => None,
            DrawImageResource::TempBuffer => Some(crate::common::CWString::new("tempbuffer")?),
            DrawImageResource::CacheBuffer(name) => {
                Some(crate::common::CWString::new(&format!("cache:{}", name))?)
            }
            DrawImageResource::ImageFile(path) => Some(crate::common::CWString::new(&format!(
                "image:{}",
                path.to_string_lossy()
            ))?),
        })
    }
}

impl FilterProcVideo {
    /// 現在の画像のデータを取得する。
    /// RGBA32bit で取得されます。
    ///
    /// # Panics
    ///
    /// `buffer` をバイト列に変換した際の長さが `width * height * 4` と一致しない場合、パニックします。
    /// 例えば[`u8`] の場合、`buffer` の長さは `width * height * 4` と一致する必要があり、
    /// [`RgbaPixel`] の場合、`buffer` の長さは `width * height` と一致する必要があります。
    ///
    /// # Note
    ///
    /// [`crate::filter::FilterPluginFlags::video`] が `true` の場合、この関数は何もせずに 0 を返します。
    pub fn get_image_data<T>(&mut self, buffer: &mut [T]) -> usize
    where
        T: Copy + FromBytes + Immutable,
    {
        if self.video_object.width == 0 || self.video_object.height == 0 {
            tracing::warn!("width or height is 0, perhaps the filter plugin is a custom object");
            return 0;
        }
        assert_eq!(
            std::mem::size_of_val(buffer),
            (self.video_object.width * self.video_object.height * 4) as usize,
            "buffer length as bytes does not match width * height * 4"
        );
        assert!(
            std::mem::align_of::<T>() >= std::mem::align_of::<aviutl2_sys::filter2::PIXEL_RGBA>(),
            "buffer alignment is not sufficient"
        );
        let width = self.video_object.width as usize;
        let height = self.video_object.height as usize;
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.get_image_data)(
                buffer.as_mut_ptr() as *mut u8 as *mut aviutl2_sys::filter2::PIXEL_RGBA
            )
        };

        width * height * 4
    }

    /// 現在の画像のデータを設定する。
    ///
    /// # Panics
    ///
    /// `data` をバイト列に変換した際の長さが `width * height * 4` と一致しない場合、パニックします。
    pub fn set_image_data<T: IntoBytes + Immutable>(
        &mut self,
        data: &[T],
        width: u32,
        height: u32,
    ) {
        let bytes = &data.as_bytes();
        assert_eq!(
            bytes.len(),
            (width * height * 4) as usize,
            "data length does not match width * height * 4"
        );
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_image_data)(
                bytes.as_ptr() as *const aviutl2_sys::filter2::PIXEL_RGBA,
                width as i32,
                height as i32,
            )
        };
    }

    /// 現在のオブジェクトの画像データのポインタをID3D11Texture2Dのポインタとして取得する。
    ///
    /// # Warning
    ///
    /// [`Self::set_image_data`] によって現在の画像が変更されるかフィルタ処理の終了まで有効です。
    pub fn get_image_texture2d(&mut self) -> *mut std::ffi::c_void {
        let inner = unsafe { &*self.inner };
        unsafe { (inner.get_image_texture2d)() }
    }

    /// 現在のフレームバッファの画像データのポインタをID3D11Texture2Dのポインタとして取得する。
    ///
    /// # Warning
    ///
    /// フィルタ処理の終了まで有効です。
    pub fn get_framebuffer_texture2d(&mut self) -> *mut std::ffi::c_void {
        let inner = unsafe { &*self.inner };
        unsafe { (inner.get_framebuffer_texture2d)() }
    }

    /// 読み取り専用の編集セクション。
    pub fn read_section(&mut self) -> &crate::generic::ReadSection {
        &self.read
    }

    /// 指定オブジェクトの画像出力項目のパラメータを取得する。
    pub fn get_output_image_param(
        &mut self,
        object: crate::generic::ObjectHandle,
        offset: f64,
    ) -> FilterProcResult<ObjectImageParam> {
        let inner = unsafe { &*self.inner };
        let mut param = aviutl2_sys::filter2::OBJECT_IMAGE_PARAM {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rx: 0.0,
            ry: 0.0,
            rz: 0.0,
            sx: 1.0,
            sy: 1.0,
            sz: 1.0,
            cx: 0.0,
            cy: 0.0,
            cz: 0.0,
            alpha: 1.0,
        };
        let ok = unsafe {
            (inner.get_output_image_param)(
                object.internal,
                offset,
                &mut param,
                std::mem::size_of::<aviutl2_sys::filter2::OBJECT_IMAGE_PARAM>() as i32,
            )
        };
        if ok {
            Ok(param.into())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 指定のレイヤーにある画像オブジェクトを取得する。
    pub fn get_image_object(
        &mut self,
        layer: u32,
        offset: f64,
    ) -> Option<crate::generic::ObjectHandle> {
        let handle = unsafe { ((*self.inner).get_image_object)(layer as _, offset) };
        if handle.is_null() {
            None
        } else {
            Some(crate::generic::ObjectHandle { internal: handle })
        }
    }

    /// 指定の画像リソースを描画する。
    ///
    /// # Arguments
    ///
    /// - `resource`: 描画する画像リソース。
    /// - `x`, `y`, `z`: 描画位置の基準座標。
    /// - `rx`, `ry`, `rz`: 描画の回転角度。（360.0で1回転）
    /// - `sx`, `sy`, `sz`: 描画の拡大率。（1.0で等倍）
    /// - `alpha`: 描画の不透明度。（0.0〜1.0/0.0=透明/1.0=不透明）
    pub fn draw_image(
        &mut self,
        resource: &DrawImageResource,
        x: f32,
        y: f32,
        z: f32,
        rx: f32,
        ry: f32,
        rz: f32,
        sx: f32,
        sy: f32,
        sz: f32,
        alpha: f32,
    ) -> FilterProcResult<()> {
        self.apply_image_param();
        let resource_str = resource.to_cw_string()?;
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.draw_image)(
                resource_str
                    .as_ref()
                    .map_or(std::ptr::null(), |s| s.as_ptr()),
                x,
                y,
                z,
                rx,
                ry,
                rz,
                sx,
                sy,
                sz,
                alpha,
            )
        };
        Ok(())
    }

    fn apply_image_param(&mut self) {
        let param = unsafe { &mut *((*self.inner).param) };
        param.x = self.param.x;
        param.y = self.param.y;
        param.z = self.param.z;
        param.rx = self.param.rx;
        param.ry = self.param.ry;
        param.rz = self.param.rz;
        param.sx = self.param.sx;
        param.sy = self.param.sy;
        param.sz = self.param.sz;
        param.cx = self.param.cx;
        param.cy = self.param.cy;
        param.cz = self.param.cz;
        param.alpha = self.param.alpha;
    }
}
