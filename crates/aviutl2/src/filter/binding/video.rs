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
    /// [`crate::filter::FilterPlugin::proc_video`] の終了時や [`Self::draw_image`]
    /// などの呼び出し前に反映されます。
    pub param: ObjectImageParam,

    pub(crate) prevent_post_effect: bool,

    pub(crate) read_section: crate::generic::ReadSection,
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

/// [`FilterProcVideo::create_image_resource`] で作成する画像リソースの種別。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateImageResource {
    /// 現在の画像データ。
    Object,
    /// 仮想バッファ。
    TempBuffer,
    /// キャッシュバッファ。
    CacheBuffer(String),
}
impl CreateImageResource {
    fn to_cw_string(
        &self,
    ) -> Result<Option<crate::common::CWString>, crate::common::NullByteError> {
        Ok(match self {
            CreateImageResource::Object => None,
            CreateImageResource::TempBuffer => Some(crate::common::CWString::new("tempbuffer")?),
            CreateImageResource::CacheBuffer(name) => {
                Some(crate::common::CWString::new(&format!("cache:{}", name))?)
            }
        })
    }
}

/// [FilterProcVideo::draw_image] のパラメーター。
#[derive(Debug, Clone, Copy)]
pub struct DrawImageParam {
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
    /// 不透明度。（0.0〜1.0/0.0=透明/1.0=不透明）
    pub alpha: f32,
}

impl Default for DrawImageParam {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rx: 0.0,
            ry: 0.0,
            rz: 0.0,
            sx: 1.0,
            sy: 1.0,
            sz: 1.0,
            alpha: 1.0,
        }
    }
}

/// 頂点データ構造体(描画色)
#[derive(Debug, Clone, Copy)]
pub struct VertexColor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// 頂点データ構造体（描画色、法線）
#[derive(Debug, Clone, Copy)]
pub struct VertexColorNorm {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

/// 頂点データ構造体（テクスチャ座標）
#[derive(Debug, Clone, Copy)]
pub struct VertexTexture {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    pub a: f32,
}

/// 頂点データ構造体（テクスチャ座標、法線）
#[derive(Debug, Clone, Copy)]
pub struct VertexTextureNorm {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    pub a: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

/// 頂点リスト。
pub enum VertexList {
    TriangleColor(Vec<[VertexColor; 3]>),
    TriangleColorNorm(Vec<[VertexColorNorm; 3]>),
    TriangleTexture(Vec<[VertexTexture; 3]>),
    TriangleTextureNorm(Vec<[VertexTextureNorm; 3]>),
    QuadColor(Vec<[VertexColor; 4]>),
    QuadColorNorm(Vec<[VertexColorNorm; 4]>),
    QuadTexture(Vec<[VertexTexture; 4]>),
    QuadTextureNorm(Vec<[VertexTextureNorm; 4]>),
}

/// 合成モード。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BlendMode {
    /// 通常。
    #[default]
    None,
    /// 加算。
    Add,
    /// 減算。
    Sub,
    /// 乗算。
    Mul,
    /// スクリーン。
    Screen,
    /// オーバーレイ。
    Overlay,
    /// 比較（明）。
    Light,
    /// 比較（暗）。
    Dark,
    /// 輝度。
    Brightness,
    /// 色差。
    Chroma,
    /// 陰影。
    Shadow,
    /// 明暗。
    LightDark,
    /// 差分。
    Diff,
}

/// サンプラーの種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SamplerMode {
    /// 領域外は透明色。
    #[default]
    Clip,
    /// 領域外は一番外側の色。
    Clamp,
    /// 領域外はループ。
    Loop,
    /// 領域外は領域を反転しながらループ。
    Mirror,
    /// 拡大縮小補間をしない（領域外は透明色）。
    Dot,
}

/// ビルボードの種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BillboardMode {
    /// 標準の向き（何もしない）。
    #[default]
    None,
    /// 横方向のみカメラに向ける。
    Side,
    /// 縦横方向のみカメラに向ける。
    Direction,
    /// カメラに向ける。
    Camera,
}

impl From<BlendMode> for aviutl2_sys::filter2::BLEND_MODE {
    fn from(value: BlendMode) -> Self {
        match value {
            BlendMode::None => Self::NONE,
            BlendMode::Add => Self::ADD,
            BlendMode::Sub => Self::SUB,
            BlendMode::Mul => Self::MUL,
            BlendMode::Screen => Self::SCREEN,
            BlendMode::Overlay => Self::OVERLAY,
            BlendMode::Light => Self::LIGHT,
            BlendMode::Dark => Self::DARK,
            BlendMode::Brightness => Self::BRIGHTNESS,
            BlendMode::Chroma => Self::CHROMA,
            BlendMode::Shadow => Self::SHADOW,
            BlendMode::LightDark => Self::LIGHT_DARK,
            BlendMode::Diff => Self::DIFF,
        }
    }
}
impl From<SamplerMode> for aviutl2_sys::filter2::SAMPLER_MODE {
    fn from(value: SamplerMode) -> Self {
        match value {
            SamplerMode::Clip => Self::CLIP,
            SamplerMode::Clamp => Self::CLAMP,
            SamplerMode::Loop => Self::LOOP,
            SamplerMode::Mirror => Self::MIRROR,
            SamplerMode::Dot => Self::DOT,
        }
    }
}
impl From<BillboardMode> for aviutl2_sys::filter2::BILLBOARD_MODE {
    fn from(value: BillboardMode) -> Self {
        match value {
            BillboardMode::None => Self::NONE,
            BillboardMode::Side => Self::SIDE,
            BillboardMode::Direction => Self::DIRECTION,
            BillboardMode::Camera => Self::CAMERA,
        }
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
        &self.read_section
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

    /// 指定の画像リソースをフレームバッファに描画する。
    ///
    /// # Arguments
    ///
    /// - `resource`: 描画する画像リソース。
    /// - `param`: 描画パラメーター。
    pub fn draw_image(
        &mut self,
        resource: &DrawImageResource,
        param: DrawImageParam,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let resource_str = resource.to_cw_string()?;
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.draw_image)(
                resource_str
                    .as_ref()
                    .map_or(std::ptr::null(), |s| s.as_ptr()),
                param.x,
                param.y,
                param.z,
                param.rx,
                param.ry,
                param.rz,
                param.sx,
                param.sy,
                param.sz,
                param.alpha,
            )
        };
        Ok(())
    }

    /// 指定の頂点リストをフレームバッファに描画する。
    pub fn draw_poly(
        &mut self,
        resource: &DrawImageResource,
        vertices: &VertexList,
    ) -> FilterProcResult<()> {
        self.apply_param();

        let resource_str = resource.to_cw_string()?;
        let inner = unsafe { &*self.inner };
        let success = match vertices {
            VertexList::TriangleColor(triangles) => {
                let flattened: Vec<VertexColor> = triangles.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_COLOR,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::TriangleColorNorm(triangles) => {
                let flattened: Vec<VertexColorNorm> = triangles.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_COLOR_NORM,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::TriangleTexture(triangles) => {
                let flattened: Vec<VertexTexture> = triangles.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_TEXTURE,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::TriangleTextureNorm(triangles) => {
                let flattened: Vec<VertexTextureNorm> =
                    triangles.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_TEXTURE_NORM,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::QuadColor(quads) => {
                let flattened: Vec<VertexColor> = quads.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::QUAD_COLOR,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::QuadColorNorm(quads) => {
                let flattened: Vec<VertexColorNorm> = quads.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::QUAD_COLOR_NORM,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::QuadTexture(quads) => {
                let flattened: Vec<VertexTexture> = quads.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::QUAD_TEXTURE,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
            VertexList::QuadTextureNorm(quads) => {
                let flattened: Vec<VertexTextureNorm> = quads.iter().flatten().copied().collect();
                unsafe {
                    (inner.draw_poly)(
                        aviutl2_sys::filter2::VERTEX_TYPE::QUAD_TEXTURE_NORM,
                        flattened.as_ptr() as *const std::ffi::c_void,
                        flattened.len() as i32,
                        resource_str
                            .as_ref()
                            .map_or(std::ptr::null(), |s| s.as_ptr()),
                    )
                }
            }
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 標準のアンカー枠を設定する。
    ///
    /// 通常は自動で設定されます。
    /// [`Self::draw_image`] などで手動で描画する場合に使います。
    /// この関数を呼び出すとこれ以降の描画処理後のフィルタ処理が中断されます。
    pub fn set_default_anchor(&mut self, width: u32, height: u32) {
        let inner = unsafe { &*self.inner };
        self.prevent_post_effect = true;
        unsafe {
            (inner.set_default_anchor)(width as i32, height as i32);
        }
    }

    /// 描画時の合成モードを設定する。
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_blend_mode)(mode.into());
        }
    }

    /// 描画時の光沢度を設定する。
    /// カメラ制御の光源設定が有効の時に利用されます。
    pub fn set_material_shine(&mut self, shininess: f32) {
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_material_shine)(shininess);
        }
    }

    /// 描画時のサンプラーを設定する。
    pub fn set_sampler_mode(&mut self, mode: SamplerMode) {
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_sampler_mode)(mode.into());
        }
    }

    /// 描画時にカメラに向けるかどうかを設定する。
    pub fn set_billboard_mode(&mut self, mode: BillboardMode) {
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_billboard_mode)(mode.into());
        }
    }

    /// 画像リソースを作成する。
    /// VRAMへデータを書き込みます。
    ///
    /// # Note
    ///
    /// - 既に同名の画像リソースが存在する場合、上書きされます。
    ///
    /// # Panics
    ///
    /// `data` をバイト列に変換した際の長さが `width * height * 4` と一致しない場合、パニックします。
    pub fn create_image_resource<T: IntoBytes + Immutable>(
        &mut self,
        resource: &CreateImageResource,
        data: &[T],
        width: u32,
        height: u32,
    ) -> FilterProcResult<()> {
        let bytes = &data.as_bytes();
        assert_eq!(
            bytes.len(),
            (width * height * 4) as usize,
            "data length does not match width * height * 4"
        );
        let inner = unsafe { &*self.inner };
        let name_cw = resource.to_cw_string()?;
        unsafe {
            (inner.create_image_resource)(
                name_cw.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                bytes.as_ptr() as *const aviutl2_sys::filter2::PIXEL_RGBA,
                width as i32,
                height as i32,
            )
        };
        Ok(())
    }

    /// フィルタ処理後の処理を中断する。
    ///
    /// [`Self::draw_poly`] や [`Self::draw_image`] などを呼び出した後に使います。
    pub fn prevent_post_effect(&mut self) {
        self.prevent_post_effect = true;
    }

    pub(crate) fn apply_param(&mut self) {
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
