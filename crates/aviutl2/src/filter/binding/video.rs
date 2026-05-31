use std::ffi::c_void;

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

impl From<(u8, u8, u8, u8)> for RgbaPixel {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}
impl From<RgbaPixel> for (u8, u8, u8, u8) {
    fn from(value: RgbaPixel) -> Self {
        (value.r, value.g, value.b, value.a)
    }
}

/// 画像リソースに書き込むピクセルフォーマット。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputImageResourcePixelFormat {
    /// DXGI_FORMAT_R8G8B8A8_UNORM。
    Rgba,
    /// DXGI_FORMAT_B8G8R8A8_UNORM。
    Bgra,
    /// DXGI_FORMAT_B8G8R8X8_UNORM。
    Bgr,
    /// DXGI_FORMAT_R16G16B16A16_UNORM。
    Pa64,
    /// DXGI_FORMAT_R16G16B16A16_FLOAT。
    Hf64,
    /// DXGI_FORMAT_YUY2。
    Yuy2,
    /// DXGI_FORMAT_R16G16B16A16_SNORM。
    Yc48,
}

impl From<InputImageResourcePixelFormat> for aviutl2_sys::filter2::INPUT_PIXEL_FORMAT {
    fn from(value: InputImageResourcePixelFormat) -> Self {
        match value {
            InputImageResourcePixelFormat::Rgba => Self::RGBA,
            InputImageResourcePixelFormat::Bgra => Self::BGRA,
            InputImageResourcePixelFormat::Bgr => Self::BGR,
            InputImageResourcePixelFormat::Pa64 => Self::PA64,
            InputImageResourcePixelFormat::Hf64 => Self::HF64,
            InputImageResourcePixelFormat::Yuy2 => Self::YUY2,
            InputImageResourcePixelFormat::Yc48 => Self::YC48,
        }
    }
}

/// 画像リソースから読み取るピクセルフォーマット。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputImageResourcePixelFormat {
    /// DXGI_FORMAT_R8G8B8A8_UNORM。
    Rgba,
    /// DXGI_FORMAT_R16G16B16A16_UNORM。
    Pa64,
    /// DXGI_FORMAT_R16G16B16A16_FLOAT。
    Hf64,
}

impl From<OutputImageResourcePixelFormat> for aviutl2_sys::filter2::OUTPUT_PIXEL_FORMAT {
    fn from(value: OutputImageResourcePixelFormat) -> Self {
        match value {
            OutputImageResourcePixelFormat::Rgba => Self::RGBA,
            OutputImageResourcePixelFormat::Pa64 => Self::PA64,
            OutputImageResourcePixelFormat::Hf64 => Self::HF64,
        }
    }
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
    /// 標準リソース。
    Resource(String),
    /// 仮想バッファ。
    TempBuffer,
    /// キャッシュバッファ。
    CacheBuffer(String),
    /// 画像ファイル。
    ImageFile(std::path::PathBuf),
}
impl std::fmt::Display for DrawImageResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawImageResource::Object => write!(f, "object"),
            DrawImageResource::Resource(name) => write!(f, "resource:{}", name),
            DrawImageResource::TempBuffer => write!(f, "tempbuffer"),
            DrawImageResource::CacheBuffer(name) => write!(f, "cache:{}", name),
            DrawImageResource::ImageFile(path) => write!(f, "image:{}", path.to_string_lossy()),
        }
    }
}

/// 書き込み先として使う画像リソース。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WritableImageResource {
    /// 現在の画像データ。
    Object,
    /// 標準リソース。
    Resource(String),
    /// 仮想バッファ。
    TempBuffer,
    /// キャッシュバッファ。
    CacheBuffer(String),
}
impl std::fmt::Display for WritableImageResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WritableImageResource::Object => write!(f, "object"),
            WritableImageResource::Resource(name) => write!(f, "resource:{}", name),
            WritableImageResource::TempBuffer => write!(f, "tempbuffer"),
            WritableImageResource::CacheBuffer(name) => write!(f, "cache:{}", name),
        }
    }
}

/// 読み込み元として使う画像リソース。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadableImageResource {
    /// 現在の画像データ。
    Object,
    /// 標準リソース。
    Resource(String),
    /// フレームバッファ。
    Framebuffer,
    /// 仮想バッファ。
    TempBuffer,
    /// キャッシュバッファ。
    CacheBuffer(String),
    /// 画像ファイル。
    ImageFile(std::path::PathBuf),
    /// 乱数バッファ。
    Random,
}
impl std::fmt::Display for ReadableImageResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadableImageResource::Object => write!(f, "object"),
            ReadableImageResource::Resource(name) => write!(f, "resource:{}", name),
            ReadableImageResource::Framebuffer => write!(f, "framebuffer"),
            ReadableImageResource::TempBuffer => write!(f, "tempbuffer"),
            ReadableImageResource::CacheBuffer(name) => write!(f, "cache:{}", name),
            ReadableImageResource::ImageFile(path) => write!(f, "image:{}", path.to_string_lossy()),
            ReadableImageResource::Random => write!(f, "random"),
        }
    }
}

/// シェーダーの出力先として使う画像リソース。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderTargetResource {
    /// 現在の画像データ。
    Object,
    /// 標準リソース。
    Resource(String),
    /// フレームバッファ。
    Framebuffer,
    /// 仮想バッファ。
    TempBuffer,
    /// キャッシュバッファ。
    CacheBuffer(String),
}
impl std::fmt::Display for ShaderTargetResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderTargetResource::Object => write!(f, "object"),
            ShaderTargetResource::Resource(name) => write!(f, "resource:{}", name),
            ShaderTargetResource::Framebuffer => write!(f, "framebuffer"),
            ShaderTargetResource::TempBuffer => write!(f, "tempbuffer"),
            ShaderTargetResource::CacheBuffer(name) => write!(f, "cache:{}", name),
        }
    }
}

/// [`ShaderTargetResource`]や[`DrawImageResource`]などの他の画像リソース型に変換するためのトレイト。
pub trait AsImageResource: std::fmt::Display {
    /// [`DrawImageResource`] に変換する。
    fn as_draw_image_resource(&self) -> Option<DrawImageResource>;
    /// [`WritableImageResource`] に変換する。
    fn as_writable_image_resource(&self) -> Option<WritableImageResource>;
    /// [`ReadableImageResource`] に変換する。
    fn as_readable_image_resource(&self) -> Option<ReadableImageResource>;
    /// [`ShaderTargetResource`] に変換する。
    fn as_shader_target_resource(&self) -> Option<ShaderTargetResource>;
}

impl<T: std::fmt::Display> AsImageResource for T {
    fn as_draw_image_resource(&self) -> Option<DrawImageResource> {
        let s = self.to_string();
        if s == "object" {
            Some(DrawImageResource::Object)
        } else if s == "tempbuffer" {
            Some(DrawImageResource::TempBuffer)
        } else if let Some(name) = s.strip_prefix("resource:") {
            Some(DrawImageResource::Resource(name.to_string()))
        } else if let Some(name) = s.strip_prefix("cache:") {
            Some(DrawImageResource::CacheBuffer(name.to_string()))
        } else {
            s.strip_prefix("image:")
                .map(|path| DrawImageResource::ImageFile(std::path::PathBuf::from(path)))
        }
    }

    fn as_writable_image_resource(&self) -> Option<WritableImageResource> {
        let s = self.to_string();
        if s == "object" {
            Some(WritableImageResource::Object)
        } else if s == "tempbuffer" {
            Some(WritableImageResource::TempBuffer)
        } else if let Some(name) = s.strip_prefix("resource:") {
            Some(WritableImageResource::Resource(name.to_string()))
        } else {
            s.strip_prefix("cache:")
                .map(|name| WritableImageResource::CacheBuffer(name.to_string()))
        }
    }

    fn as_readable_image_resource(&self) -> Option<ReadableImageResource> {
        let s = self.to_string();
        if s == "object" {
            Some(ReadableImageResource::Object)
        } else if s == "framebuffer" {
            Some(ReadableImageResource::Framebuffer)
        } else if s == "tempbuffer" {
            Some(ReadableImageResource::TempBuffer)
        } else if s == "random" {
            Some(ReadableImageResource::Random)
        } else if let Some(name) = s.strip_prefix("resource:") {
            Some(ReadableImageResource::Resource(name.to_string()))
        } else if let Some(name) = s.strip_prefix("cache:") {
            Some(ReadableImageResource::CacheBuffer(name.to_string()))
        } else {
            s.strip_prefix("image:")
                .map(|path| ReadableImageResource::ImageFile(std::path::PathBuf::from(path)))
        }
    }

    fn as_shader_target_resource(&self) -> Option<ShaderTargetResource> {
        let s = self.to_string();
        if s == "object" {
            Some(ShaderTargetResource::Object)
        } else if s == "framebuffer" {
            Some(ShaderTargetResource::Framebuffer)
        } else if s == "tempbuffer" {
            Some(ShaderTargetResource::TempBuffer)
        } else if let Some(name) = s.strip_prefix("resource:") {
            Some(ShaderTargetResource::Resource(name.to_string()))
        } else {
            s.strip_prefix("cache:")
                .map(|name| ShaderTargetResource::CacheBuffer(name.to_string()))
        }
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

impl From<VertexColor> for aviutl2_sys::filter2::VERTEX_COLOR {
    fn from(value: VertexColor) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl From<VertexColorNorm> for aviutl2_sys::filter2::VERTEX_COLOR_NORM {
    fn from(value: VertexColorNorm) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
            vx: value.vx,
            vy: value.vy,
            vz: value.vz,
        }
    }
}

impl From<VertexTexture> for aviutl2_sys::filter2::VERTEX_TEXTURE {
    fn from(value: VertexTexture) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            u: value.u,
            v: value.v,
            a: value.a,
        }
    }
}

impl From<VertexTextureNorm> for aviutl2_sys::filter2::VERTEX_TEXTURE_NORM {
    fn from(value: VertexTextureNorm) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            u: value.u,
            v: value.v,
            a: value.a,
            vx: value.vx,
            vy: value.vy,
            vz: value.vz,
        }
    }
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

/// 出力ブレンドの種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BlendStateMode {
    /// 出力をそのままコピー。
    #[default]
    Copy,
    /// α値のみを乗算。
    Mask,
    /// 出力をアルファブレンド。
    Draw,
    /// 出力を加算合成。
    Add,
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
impl From<BlendStateMode> for aviutl2_sys::filter2::BLEND_STATE_MODE {
    fn from(value: BlendStateMode) -> Self {
        match value {
            BlendStateMode::Copy => Self::COPY,
            BlendStateMode::Mask => Self::MASK,
            BlendStateMode::Draw => Self::DRAW,
            BlendStateMode::Add => Self::ADD,
        }
    }
}

fn vertex_list_as_raw(
    vertices: &VertexList,
    f: impl FnOnce(aviutl2_sys::filter2::VERTEX_TYPE, *const c_void, i32) -> bool,
) -> bool {
    match vertices {
        VertexList::TriangleColor(triangles) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_COLOR> = triangles
                .iter()
                .flatten()
                .copied()
                .map(Into::into)
                .collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_COLOR,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::TriangleColorNorm(triangles) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_COLOR_NORM> = triangles
                .iter()
                .flatten()
                .copied()
                .map(Into::into)
                .collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_COLOR_NORM,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::TriangleTexture(triangles) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_TEXTURE> = triangles
                .iter()
                .flatten()
                .copied()
                .map(Into::into)
                .collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_TEXTURE,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::TriangleTextureNorm(triangles) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_TEXTURE_NORM> = triangles
                .iter()
                .flatten()
                .copied()
                .map(Into::into)
                .collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::TRIANGLE_TEXTURE_NORM,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::QuadColor(quads) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_COLOR> =
                quads.iter().flatten().copied().map(Into::into).collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::QUAD_COLOR,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::QuadColorNorm(quads) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_COLOR_NORM> =
                quads.iter().flatten().copied().map(Into::into).collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::QUAD_COLOR_NORM,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::QuadTexture(quads) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_TEXTURE> =
                quads.iter().flatten().copied().map(Into::into).collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::QUAD_TEXTURE,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
        VertexList::QuadTextureNorm(quads) => {
            let flattened: Vec<aviutl2_sys::filter2::VERTEX_TEXTURE_NORM> =
                quads.iter().flatten().copied().map(Into::into).collect();
            f(
                aviutl2_sys::filter2::VERTEX_TYPE::QUAD_TEXTURE_NORM,
                flattened.as_ptr() as *const c_void,
                flattened.len() as i32,
            )
        }
    }
}

fn resource_ptr_list(
    resources: &[ReadableImageResource],
) -> Result<
    (
        Vec<crate::common::CWString>,
        Vec<aviutl2_sys::common::LPCWSTR>,
    ),
    FilterProcError,
> {
    let strings = resources
        .iter()
        .map(|resource| crate::common::CWString::new(&resource.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
    Ok((strings, ptrs))
}

fn target_ptr_list(
    resources: &[ShaderTargetResource],
) -> Result<
    (
        Vec<crate::common::CWString>,
        Vec<aviutl2_sys::common::LPCWSTR>,
    ),
    FilterProcError,
> {
    let strings = resources
        .iter()
        .map(|resource| crate::common::CWString::new(&resource.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let ptrs = strings.iter().map(|s| s.as_ptr()).collect();
    Ok((strings, ptrs))
}

fn image_data_byte_len(pitch: u32, height: u32) -> FilterProcResult<usize> {
    (pitch as usize)
        .checked_mul(height as usize)
        .ok_or(FilterProcError::ValueOutOfRange)
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
        let mut param = std::mem::MaybeUninit::<aviutl2_sys::filter2::OBJECT_IMAGE_PARAM>::uninit();
        unsafe {
            let ok = (inner.get_output_image_param)(
                object.internal,
                offset,
                param.as_mut_ptr(),
                std::mem::size_of::<aviutl2_sys::filter2::OBJECT_IMAGE_PARAM>() as i32,
            );
            if ok {
                Ok(param.assume_init().into())
            } else {
                Err(FilterProcError::ApiCallFailed)
            }
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
        let resource_str = crate::common::CWString::new(&resource.to_string())?;
        let inner = unsafe { &*self.inner };
        let success = unsafe {
            (inner.draw_image)(
                resource_str.as_ptr(),
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
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 指定の頂点リストをフレームバッファに描画する。
    pub fn draw_poly(
        &mut self,
        vertices: &VertexList,
        resource: Option<&DrawImageResource>,
    ) -> FilterProcResult<()> {
        self.apply_param();

        let resource_str = crate::common::CWString::new(
            &resource.map_or_else(|| "tempbuffer".to_string(), |r| r.to_string()),
        )?;
        let inner = unsafe { &*self.inner };
        let success = vertex_list_as_raw(vertices, |vertex_type, vertex_list, vertex_num| unsafe {
            (inner.draw_poly)(vertex_type, vertex_list, vertex_num, resource_str.as_ptr())
        });
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

    /// 描画時に裏面を非表示にするかを設定する。
    pub fn set_culling_state(&mut self, culling: bool) {
        let inner = unsafe { &*self.inner };
        unsafe {
            (inner.set_culling_state)(culling);
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
        resource: &WritableImageResource,
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
        let name_cw = crate::common::CWString::new(&resource.to_string())?;
        unsafe {
            (inner.create_image_resource)(
                name_cw.as_ptr(),
                bytes.as_ptr() as *const aviutl2_sys::filter2::PIXEL_RGBA,
                width as i32,
                height as i32,
            )
        };
        Ok(())
    }

    /// 指定の画像リソースのD3D画像リソースのポインタを取得する。
    pub fn get_image_resource_texture2d(
        &mut self,
        resource: &ReadableImageResource,
    ) -> FilterProcResult<*mut c_void> {
        let inner = unsafe { &*self.inner };
        let resource_cw = crate::common::CWString::new(&resource.to_string())?;
        let ptr = unsafe { (inner.get_image_resource_texture2d)(resource_cw.as_ptr()) };
        if ptr.is_null() {
            Err(FilterProcError::ApiCallFailed)
        } else {
            Ok(ptr)
        }
    }

    /// 指定の画像リソースのサイズを取得する。
    pub fn get_image_resource_size(
        &mut self,
        resource: &ReadableImageResource,
    ) -> FilterProcResult<(u32, u32)> {
        let inner = unsafe { &*self.inner };
        let resource_cw = crate::common::CWString::new(&resource.to_string())?;
        let mut width = 0;
        let mut height = 0;
        let success = unsafe {
            (inner.get_image_resource_size)(resource_cw.as_ptr(), &mut width, &mut height)
        };
        if success {
            Ok((width as u32, height as u32))
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 画像リソースをコピーする。
    pub fn copy_image_resource(
        &mut self,
        src_resource: &ReadableImageResource,
        dst_resource: &WritableImageResource,
    ) -> FilterProcResult<()> {
        let inner = unsafe { &*self.inner };
        let dst_resource = crate::common::CWString::new(&dst_resource.to_string())?;
        let src_resource = crate::common::CWString::new(&src_resource.to_string())?;
        let success =
            unsafe { (inner.copy_image_resource)(dst_resource.as_ptr(), src_resource.as_ptr()) };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 画像リソースをクリアする。
    pub fn clear_image_resource(
        &mut self,
        resource: &WritableImageResource,
        color: RgbaPixel,
    ) -> FilterProcResult<()> {
        let inner = unsafe { &*self.inner };
        let resource = crate::common::CWString::new(&resource.to_string())?;
        let color = aviutl2_sys::filter2::PIXEL_RGBA {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        };
        let success = unsafe { (inner.clear_image_resource)(resource.as_ptr(), color) };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 画像リソースから指定フォーマットの画像データを取得する。
    ///
    /// `buffer` は少なくとも `pitch * height` バイト必要です。
    pub fn get_image_resource_data(
        &mut self,
        resource: &ReadableImageResource,
        buffer: &mut [u8],
        width: u32,
        height: u32,
        pitch: u32,
        format: OutputImageResourcePixelFormat,
    ) -> FilterProcResult<()> {
        let min_len = image_data_byte_len(pitch, height)?;
        assert!(
            buffer.len() >= min_len,
            "buffer length must be at least pitch * height"
        );
        let inner = unsafe { &*self.inner };
        let resource = crate::common::CWString::new(&resource.to_string())?;
        let success = unsafe {
            (inner.get_image_resource_data)(
                resource.as_ptr(),
                buffer.as_mut_ptr() as *mut c_void,
                width as i32,
                height as i32,
                pitch as i32,
                format.into(),
            )
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 画像リソースに指定フォーマットの画像データを設定する。
    ///
    /// `data` をバイト列に変換した長さは少なくとも `pitch * height` バイト必要です。
    pub fn set_image_resource_data<T: IntoBytes + Immutable>(
        &mut self,
        resource: &WritableImageResource,
        data: &[T],
        width: u32,
        height: u32,
        pitch: u32,
        format: InputImageResourcePixelFormat,
    ) -> FilterProcResult<()> {
        let bytes = data.as_bytes();
        let min_len = image_data_byte_len(pitch, height)?;
        assert!(
            bytes.len() >= min_len,
            "data length must be at least pitch * height"
        );
        let inner = unsafe { &*self.inner };
        let resource = crate::common::CWString::new(&resource.to_string())?;
        let success = unsafe {
            (inner.set_image_resource_data)(
                resource.as_ptr(),
                bytes.as_ptr() as *const c_void,
                width as i32,
                height as i32,
                pitch as i32,
                format.into(),
            )
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 指定の画像リソースを描画先の画像リソースに描画する。
    pub fn draw_image_to_resource(
        &mut self,
        src_resource: &DrawImageResource,
        dst_resource: &WritableImageResource,
        param: DrawImageParam,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let inner = unsafe { &*self.inner };
        let dst_resource = crate::common::CWString::new(&dst_resource.to_string())?;
        let src_resource = crate::common::CWString::new(&src_resource.to_string())?;
        let success = unsafe {
            (inner.draw_image_to_resource)(
                dst_resource.as_ptr(),
                src_resource.as_ptr(),
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
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 指定の頂点リストのポリゴンを描画先の画像リソースに描画する。
    pub fn draw_poly_to_resource(
        &mut self,
        dst_resource: &WritableImageResource,
        vertices: &VertexList,
        src_resource: Option<&DrawImageResource>,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let inner = unsafe { &*self.inner };
        let dst_resource = crate::common::CWString::new(&dst_resource.to_string())?;
        let src_resource = crate::common::CWString::new(
            &src_resource.map_or_else(|| "tempbuffer".to_string(), |r| r.to_string()),
        )?;
        let success = vertex_list_as_raw(vertices, |vertex_type, vertex_list, vertex_num| unsafe {
            (inner.draw_poly_to_resource)(
                dst_resource.as_ptr(),
                vertex_type,
                vertex_list as *mut c_void,
                vertex_num,
                src_resource.as_ptr(),
            )
        });
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// ピクセルシェーダーを実行する。
    ///
    /// シェーダー側とメモリレイアウトを合わせるため、`constant` を構造体で渡す場合は `#[repr(C)]`
    /// を推奨します。
    pub fn exec_pixelshader<T: Copy>(
        &mut self,
        cso_file: &str,
        target: &ShaderTargetResource,
        resources: &[ReadableImageResource],
        constant: T,
        blend_state: Option<*mut c_void>,
        sampler_state: Option<*mut c_void>,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let inner = unsafe { &*self.inner };
        let cso_file = crate::common::CWString::new(cso_file)?;
        let target = crate::common::CWString::new(&target.to_string())?;
        let (_resource_strings, mut resource_ptrs) = resource_ptr_list(resources)?;
        let constant_ptr = Box::new(constant);
        let success = unsafe {
            (inner.exec_pixelshader_file)(
                cso_file.as_ptr(),
                target.as_ptr(),
                if resource_ptrs.is_empty() {
                    std::ptr::null_mut()
                } else {
                    resource_ptrs.as_mut_ptr()
                },
                resource_ptrs.len() as i32,
                constant_ptr.as_ref() as *const T as *mut c_void,
                std::mem::size_of::<T>() as i32,
                blend_state.unwrap_or(std::ptr::null_mut()),
                sampler_state.unwrap_or(std::ptr::null_mut()),
            )
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// メモリ上のコンパイル済みピクセルシェーダーを実行する。
    ///
    /// シェーダー側とメモリレイアウトを合わせるため、`constant` を構造体で渡す場合は `#[repr(C)]`
    /// を推奨します。
    pub fn exec_pixelshader_data<T: Copy>(
        &mut self,
        data: &[u8],
        target: &ShaderTargetResource,
        resources: &[ReadableImageResource],
        constant: T,
        blend_state: Option<*mut c_void>,
        sampler_state: Option<*mut c_void>,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let inner = unsafe { &*self.inner };
        let data_size = i32::try_from(data.len()).map_err(|_| FilterProcError::ValueOutOfRange)?;
        let target = crate::common::CWString::new(&target.to_string())?;
        let (_resource_strings, mut resource_ptrs) = resource_ptr_list(resources)?;
        let constant_ptr = Box::new(constant);
        let success = unsafe {
            (inner.exec_pixelshader_data)(
                data.as_ptr(),
                data_size,
                target.as_ptr(),
                if resource_ptrs.is_empty() {
                    std::ptr::null_mut()
                } else {
                    resource_ptrs.as_mut_ptr()
                },
                resource_ptrs.len() as i32,
                constant_ptr.as_ref() as *const T as *mut c_void,
                std::mem::size_of::<T>() as i32,
                blend_state.unwrap_or(std::ptr::null_mut()),
                sampler_state.unwrap_or(std::ptr::null_mut()),
            )
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// コンピュートシェーダーを実行する。
    ///
    /// シェーダー側とメモリレイアウトを合わせるため、`constant` を構造体で渡す場合は `#[repr(C)]`
    /// を推奨します。
    pub fn exec_computeshader<T: Copy>(
        &mut self,
        cso_file: &str,
        targets: &[ShaderTargetResource],
        resources: &[ReadableImageResource],
        constant: T,
        count: [u32; 3],
        sampler_state: Option<*mut c_void>,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let inner = unsafe { &*self.inner };
        let cso_file = crate::common::CWString::new(cso_file)?;
        let (_target_strings, mut target_ptrs) = target_ptr_list(targets)?;
        let (_resource_strings, mut resource_ptrs) = resource_ptr_list(resources)?;
        let constant_ptr = Box::new(constant);
        let success = unsafe {
            (inner.exec_computeshader_file)(
                cso_file.as_ptr(),
                if target_ptrs.is_empty() {
                    std::ptr::null_mut()
                } else {
                    target_ptrs.as_mut_ptr()
                },
                target_ptrs.len() as i32,
                if resource_ptrs.is_empty() {
                    std::ptr::null_mut()
                } else {
                    resource_ptrs.as_mut_ptr()
                },
                resource_ptrs.len() as i32,
                constant_ptr.as_ref() as *const T as *mut c_void,
                std::mem::size_of::<T>() as i32,
                count[0] as i32,
                count[1] as i32,
                count[2] as i32,
                sampler_state.unwrap_or(std::ptr::null_mut()),
            )
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// メモリ上のコンパイル済みコンピュートシェーダーを実行する。
    ///
    /// シェーダー側とメモリレイアウトを合わせるため、`constant` を構造体で渡す場合は `#[repr(C)]`
    /// を推奨します。
    pub fn exec_computeshader_data<T: Copy>(
        &mut self,
        data: &[u8],
        targets: &[ShaderTargetResource],
        resources: &[ReadableImageResource],
        constant: T,
        count: [u32; 3],
        sampler_state: Option<*mut c_void>,
    ) -> FilterProcResult<()> {
        self.apply_param();
        let inner = unsafe { &*self.inner };
        let data_size = i32::try_from(data.len()).map_err(|_| FilterProcError::ValueOutOfRange)?;
        let (_target_strings, mut target_ptrs) = target_ptr_list(targets)?;
        let (_resource_strings, mut resource_ptrs) = resource_ptr_list(resources)?;
        let constant_ptr = Box::new(constant);
        let success = unsafe {
            (inner.exec_computeshader_data)(
                data.as_ptr(),
                data_size,
                if target_ptrs.is_empty() {
                    std::ptr::null_mut()
                } else {
                    target_ptrs.as_mut_ptr()
                },
                target_ptrs.len() as i32,
                if resource_ptrs.is_empty() {
                    std::ptr::null_mut()
                } else {
                    resource_ptrs.as_mut_ptr()
                },
                resource_ptrs.len() as i32,
                constant_ptr.as_ref() as *const T as *mut c_void,
                std::mem::size_of::<T>() as i32,
                count[0] as i32,
                count[1] as i32,
                count[2] as i32,
                sampler_state.unwrap_or(std::ptr::null_mut()),
            )
        };
        if success {
            Ok(())
        } else {
            Err(FilterProcError::ApiCallFailed)
        }
    }

    /// 定義済みのD3Dの出力ブレンドのリソースのポインタを取得する。
    pub fn get_blend_state(&mut self, mode: BlendStateMode) -> Option<*mut c_void> {
        let inner = unsafe { &*self.inner };
        let ptr = unsafe { (inner.get_blend_state)(mode.into()) };
        (!ptr.is_null()).then_some(ptr)
    }

    /// 定義済みのD3Dのサンプラーのリソースのポインタを取得する。
    pub fn get_sampler_state(&mut self, mode: SamplerMode) -> Option<*mut c_void> {
        let inner = unsafe { &*self.inner };
        let ptr = unsafe { (inner.get_sampler_state)(mode.into()) };
        (!ptr.is_null()).then_some(ptr)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_image_resources_to_supported_resource_types() {
        let readable_image = ReadableImageResource::ImageFile(std::path::PathBuf::from("foo.png"));
        assert_eq!(
            readable_image.as_draw_image_resource(),
            Some(DrawImageResource::ImageFile(std::path::PathBuf::from(
                "foo.png"
            )))
        );
        assert_eq!(readable_image.as_writable_image_resource(), None);
        assert_eq!(readable_image.as_shader_target_resource(), None);

        let shader_target = ShaderTargetResource::Framebuffer;
        assert_eq!(
            shader_target.as_readable_image_resource(),
            Some(ReadableImageResource::Framebuffer)
        );
        assert_eq!(
            shader_target.as_shader_target_resource(),
            Some(ShaderTargetResource::Framebuffer)
        );
        assert_eq!(shader_target.as_draw_image_resource(), None);
        assert_eq!(shader_target.as_writable_image_resource(), None);
    }

    #[test]
    fn converts_common_image_resources_to_all_compatible_resource_types() {
        let cache = DrawImageResource::CacheBuffer("buf".to_string());
        assert_eq!(
            cache.as_writable_image_resource(),
            Some(WritableImageResource::CacheBuffer("buf".to_string()))
        );
        assert_eq!(
            cache.as_readable_image_resource(),
            Some(ReadableImageResource::CacheBuffer("buf".to_string()))
        );
        assert_eq!(
            cache.as_shader_target_resource(),
            Some(ShaderTargetResource::CacheBuffer("buf".to_string()))
        );

        let random = ReadableImageResource::Random;
        assert_eq!(random.as_draw_image_resource(), None);
        assert_eq!(random.as_writable_image_resource(), None);
        assert_eq!(random.as_shader_target_resource(), None);
        assert_eq!(
            random.as_readable_image_resource(),
            Some(ReadableImageResource::Random)
        );
    }

    #[allow(dead_code)]
    fn smoke_new_filter2_api(video: &mut FilterProcVideo) -> FilterProcResult<()> {
        let writable = WritableImageResource::Resource("dst".to_string());
        let readable = ReadableImageResource::Resource("src".to_string());
        let drawable = DrawImageResource::Resource("src".to_string());
        let target = ShaderTargetResource::Resource("target".to_string());
        let vertices = VertexList::TriangleColor(vec![[
            VertexColor {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            VertexColor {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
            VertexColor {
                x: 0.0,
                y: 1.0,
                z: 0.0,
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
        ]]);
        let constant = [0u8; 16];

        video.set_culling_state(true);
        let _ = video.get_image_resource_texture2d(&readable)?;
        video.copy_image_resource(&readable, &writable)?;
        video.clear_image_resource(&writable, RgbaPixel::default())?;
        video.draw_image_to_resource(&drawable, &writable, DrawImageParam::default())?;
        video.draw_poly_to_resource(&writable, &vertices, Some(&drawable))?;
        let blend_state = video.get_blend_state(BlendStateMode::Draw);
        let sampler_state = video.get_sampler_state(SamplerMode::Clamp);
        video.exec_pixelshader(
            "shader.cso",
            &target,
            std::slice::from_ref(&readable),
            Some(&constant),
            blend_state,
            sampler_state,
        )?;
        video.exec_computeshader(
            "compute.cso",
            std::slice::from_ref(&target),
            std::slice::from_ref(&readable),
            Some(&constant),
            [1, 1, 1],
            sampler_state,
        )?;
        Ok(())
    }
}
