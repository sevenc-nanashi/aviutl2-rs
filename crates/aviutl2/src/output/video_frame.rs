use crate::{
    common::f16,
    output::{VideoOutputInfo, Yc48},
};
use std::{
    ops::Deref,
    sync::{Arc, atomic::AtomicUsize},
};

/// 動画フレームを表すトレイト。
/// aviutl2-rsでは、このトレイトを実装した型で動画フレームのフォーマットを指定します。
pub trait FromRawVideoFrame {
    /// 動画フレームのフォーマットを表す定数。
    const FORMAT: u32;

    /// 動画フレームのフォーマットが出力情報に適合するかをチェックする。
    /// 例えば、[`Yuy2VideoFrame`]（YUV
    /// 4:2:2）を使用する場合は、出力情報の幅と高さが偶数であることを確認します。
    fn check(video: &VideoOutputInfo) -> Result<(), String>;

    /// 動画フレームを生のポインタから取得する。
    ///
    /// # Safety
    /// func_get_videoの戻り値のポインタのみが許容される。
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        last_frame_id: Arc<AtomicUsize>,
        frame_id: usize,
    ) -> Self;
}

duplicate::duplicate! {
    [
        Name                Type                   Doc;
        [RgbVideoFrame]     [(u8, u8, u8)]         ["(u8, u8, u8) で表されるRGBの動画フレーム。"];
        [Yuy2VideoFrame]    [(u8, u8, u8, u8)]     ["(u8, u8, u8, u8) で表されるYUV 4:2:2の動画フレーム。"];
        [Hf64VideoFrame]    [(f16, f16, f16, f16)] ["(f16, f16, f16, f16) で表されるRGBAの動画フレーム。"];
        [Yc48VideoFrame]    [Yc48]                 ["YC48形式の動画フレーム。"];
        [Pa64VideoFrame]    [(u16, u16, u16, u16)] ["(u16, u16, u16, u16) で表されるRGBAの動画フレーム。"];

        [RawBgrVideoFrame]  [u8]                   ["生のBGR24形式の動画フレームデータ。"];
        [RawYuy2VideoFrame] [u8]                   ["生のYUV 4:2:2形式の動画フレームデータ。"];
        [RawHf64VideoFrame] [f16]                  ["生のDXGI_FORMAT_R16G16B16A16_FLOAT（乗算済みα）形式の動画フレームデータ。"];
        [RawYc48VideoFrame] [i16]                  ["生のYC48形式の動画フレームデータ。"];
        [RawPa64VideoFrame] [u16]                  ["生のDXGI_FORMAT_R16G16B16A16_UNORM（乗算済みα）形式の動画フレームデータ。"];
    ]
    #[doc = Doc]
    #[derive(Debug, Clone)]
    pub struct Name {
        pub data: Vec<Type>,
    }
    impl Deref for Name {
        type Target = [Type];

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }
}

duplicate::duplicate! {
    [
        Name                        OwnedName           ParsedName       Type                   Doc;
        [BorrowedRawBgrVideoFrame]  [RawBgrVideoFrame]  [RgbVideoFrame]  [u8]                   ["生のBGR24形式の動画フレームデータ。"];
        [BorrowedRawYuy2VideoFrame] [RawYuy2VideoFrame] [Yuy2VideoFrame] [u8]                   ["生のYUV 4:2:2形式の動画フレームデータ。"];
        [BorrowedRawHf64VideoFrame] [RawHf64VideoFrame] [Hf64VideoFrame] [f16]                  ["生のDXGI_FORMAT_R16G16B16A16_FLOAT（乗算済みα）形式の動画フレームデータ。"];
        [BorrowedRawYc48VideoFrame] [RawYc48VideoFrame] [Yc48VideoFrame] [i16]                  ["生のYC48形式の動画フレームデータ。"];
        [BorrowedRawPa64VideoFrame] [RawPa64VideoFrame] [Pa64VideoFrame] [u16]                  ["生のDXGI_FORMAT_R16G16B16A16_UNORM（乗算済みα）形式の動画フレームデータ。"];
    ]
    #[doc = Doc]
    #[doc = concat!("[`", stringify!(OwnedName), "`]や[`", stringify!(ParsedName), "`]とは違い、フレームデータを所有しません。")]
    #[derive(Debug)]
    pub struct Name {
        data: *const Type,
        length: usize,

        last_frame_id: Arc<AtomicUsize>,
        frame_id: usize,

        info: VideoOutputInfo,
    }
    impl Name {
        /// この型が参照するデータの長さを返します。
        pub fn len(&self) -> usize {
            self.length
        }

        /// この型が参照するデータが空かどうかを返します。
        ///
        /// # Note
        ///
        /// 常に`false`を返します。
        pub fn is_empty(&self) -> bool {
            false
        }

        /// この型の内部のポインタを返します。
        pub fn as_ptr(&self) -> *const Type {
            self.data
        }

        /// この型がまだ有効かどうかを返します。
        pub fn is_valid(&self) -> bool {
            self.last_frame_id.load(std::sync::atomic::Ordering::SeqCst) == self.frame_id
        }

        /// この型の持っているデータをスライスとして返します。
        ///
        /// # Panics
        ///
        /// 次のフレーム取得が行われた後や、[`OutputInfo`][`super::OutputInfo`]が破棄された後に呼び出すとパニックになります。
        pub fn as_slice(&self) -> &[Type] {
            assert!(
                self.is_valid(),
                "The frame data has been invalidated. This can happen if a new frame is fetched"
            );
            unsafe { self.as_slice_unchecked() }
        }

        /// この型の持っているデータをスライスとして返します。
        ///
        /// # Safety
        ///
        /// 次のフレーム取得が行われた後や、[`OutputInfo`][`super::OutputInfo`]が破棄された後に呼び出すと未定義動作になります。
        pub unsafe fn as_slice_unchecked(&self) -> &[Type] {
            unsafe { std::slice::from_raw_parts(self.data, self.length) }
        }

        /// この型の持っているデータを所有する型に変換します。
        ///
        /// # Panics
        ///
        /// 次のフレーム取得が行われた後や、[`OutputInfo`][`super::OutputInfo`]が破棄された後に呼び出すとパニックになります。
        pub fn to_owned(&self) -> OwnedName {
            assert!(
                self.is_valid(),
                "The frame data has been invalidated. This can happen if a new frame is fetched"
            );
            unsafe { self.to_owned_unchecked() }
        }

        /// この型の持っているデータを所有する型に変換します。
        ///
        /// # Safety
        ///
        /// 次のフレーム取得が行われた後や、[`OutputInfo`][`super::OutputInfo`]が破棄された後に呼び出すと未定義動作になります。
        pub unsafe fn to_owned_unchecked(&self) -> OwnedName {
            OwnedName {
                data: unsafe { std::slice::from_raw_parts(self.data, self.length).to_vec() },
            }
        }

        /// この型の持っているデータを解析した型に変換します。
        ///
        /// # Panics
        ///
        /// 次のフレーム取得が行われた後や、[`OutputInfo`][`super::OutputInfo`]が破棄された後に呼び出すとパニックになります。
        pub fn to_parsed(&self) -> ParsedName {
            assert!(
                self.is_valid(),
                "The frame data has been invalidated. This can happen if a new frame is fetched"
            );
            unsafe { self.to_parsed_unchecked() }
        }

        /// この型の持っているデータを解析した型に変換します。
        ///
        /// # Safety
        ///
        /// 次のフレーム取得が行われた後や、[`OutputInfo`][`super::OutputInfo`]が破棄された後に呼び出すと未定義動作になります。
        pub unsafe fn to_parsed_unchecked(&self) -> ParsedName {
            #[allow(clippy::unnecessary_cast)]
            unsafe { ParsedName::from_raw(&self.info, self.data as *const u8, self.last_frame_id.clone(), self.frame_id)}
        }
    }
}

impl FromRawVideoFrame for RgbVideoFrame {
    const FORMAT: u32 = aviutl2_sys::common::BI_RGB;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
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
    const FORMAT: u32 = aviutl2_sys::common::BI_YUY2;

    fn check(video: &VideoOutputInfo) -> Result<(), String> {
        if !video.width.is_multiple_of(2) || !video.height.is_multiple_of(2) {
            return Err("YUY2 format requires even width and height".to_string());
        }
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
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

impl FromRawVideoFrame for Hf64VideoFrame {
    const FORMAT: u32 = aviutl2_sys::common::BI_HF64;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        let frame_data_ptr = frame_data_ptr as *const u16;
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 8 bytes (RGBA)
                let pixel_r = unsafe { *frame_data_ptr.add(i * 4) };
                let pixel_g = unsafe { *frame_data_ptr.add(i * 4 + 1) };
                let pixel_b = unsafe { *frame_data_ptr.add(i * 4 + 2) };
                let pixel_a = unsafe { *frame_data_ptr.add(i * 4 + 3) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((
                        f16::from_bits(pixel_r),
                        f16::from_bits(pixel_g),
                        f16::from_bits(pixel_b),
                        f16::from_bits(pixel_a),
                    ));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for Yc48VideoFrame {
    const FORMAT: u32 = aviutl2_sys::common::BI_YC48;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        let frame_data_ptr = frame_data_ptr as *const i16;
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 6 bytes (YCbCr)
                let pixel_y = unsafe { *frame_data_ptr.add(i * 3) };
                let pixel_cr = unsafe { *frame_data_ptr.add(i * 3 + 1) };
                let pixel_cb = unsafe { *frame_data_ptr.add(i * 3 + 2) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write(Yc48 {
                        y: pixel_y,
                        cr: pixel_cr,
                        cb: pixel_cb,
                    });
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}
impl FromRawVideoFrame for Pa64VideoFrame {
    const FORMAT: u32 = aviutl2_sys::common::BI_PA64;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let mut frame_buffer = Vec::with_capacity((video.width * video.height) as usize);
        let frame_data_writer = frame_buffer.spare_capacity_mut();
        let frame_data_ptr = frame_data_ptr as *const u16;
        for y in 0..video.height as usize {
            for x in 0..video.width as usize {
                let i = y * video.width as usize + x;
                // Each pixel is represented by 8 bytes (RGBA)
                let pixel_r = unsafe { *frame_data_ptr.add(i * 4) };
                let pixel_g = unsafe { *frame_data_ptr.add(i * 4 + 1) };
                let pixel_b = unsafe { *frame_data_ptr.add(i * 4 + 2) };
                let pixel_a = unsafe { *frame_data_ptr.add(i * 4 + 3) };
                frame_data_writer[(video.height as usize - 1 - y) * video.width as usize + x]
                    .write((pixel_r, pixel_g, pixel_b, pixel_a));
            }
        }
        unsafe {
            frame_buffer.set_len((video.width * video.height) as usize);
        }

        Self { data: frame_buffer }
    }
}

#[duplicate::duplicate_item(
    Name                Type  elms FMT;
    [RawBgrVideoFrame]  [u8]  [3]  [aviutl2_sys::common::BI_RGB];
    [RawYuy2VideoFrame] [u8]  [2]  [aviutl2_sys::common::BI_YUY2];
    [RawHf64VideoFrame] [f16] [4]  [aviutl2_sys::common::BI_HF64];
    [RawYc48VideoFrame] [i16] [3]  [aviutl2_sys::common::BI_YC48];
    [RawPa64VideoFrame] [u16] [4]  [aviutl2_sys::common::BI_PA64];
)]
impl FromRawVideoFrame for Name {
    const FORMAT: u32 = FMT;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let frame_buffer = unsafe {
            #[allow(clippy::unnecessary_cast)]
            std::slice::from_raw_parts(
                frame_data_ptr as *const Type,
                (video.width * video.height * elms) as usize,
            )
            .to_owned()
        };

        Self { data: frame_buffer }
    }
}

#[duplicate::duplicate_item(
    Name                        Type  elms FMT;
    [BorrowedRawBgrVideoFrame]  [u8]  [3]  [aviutl2_sys::common::BI_RGB];
    [BorrowedRawYuy2VideoFrame] [u8]  [2]  [aviutl2_sys::common::BI_YUY2];
    [BorrowedRawHf64VideoFrame] [f16] [4]  [aviutl2_sys::common::BI_HF64];
    [BorrowedRawYc48VideoFrame] [i16] [3]  [aviutl2_sys::common::BI_YC48];
    [BorrowedRawPa64VideoFrame] [u16] [4]  [aviutl2_sys::common::BI_PA64];
)]
impl FromRawVideoFrame for Name {
    const FORMAT: u32 = FMT;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        last_frame_id: Arc<AtomicUsize>,
        frame_id: usize,
    ) -> Self {
        let length = (video.width * video.height * elms) as usize;

        Self {
            data: frame_data_ptr as _,
            length,
            info: video.clone(),
            last_frame_id,
            frame_id,
        }
    }
}

#[cfg(feature = "image")]
impl FromRawVideoFrame for image::RgbImage {
    const FORMAT: u32 = aviutl2_sys::common::BI_RGB;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let mut buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 3) as usize)
                .to_owned()
        };
        crate::utils::bgr_to_rgb_bytes(&mut buffer);
        crate::utils::flip_vertical(&mut buffer, video.width as usize * 3, video.height as usize);
        image::RgbImage::from_raw(video.width, video.height, buffer).unwrap()
    }
}

#[cfg(feature = "image")]
impl FromRawVideoFrame for image::ImageBuffer<image::Rgba<u16>, Vec<u16>> {
    const FORMAT: u32 = aviutl2_sys::common::BI_PA64;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let frame_data_ptr = frame_data_ptr as *const u16;
        let buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 4) as usize)
                .to_owned()
        };
        image::ImageBuffer::from_raw(video.width, video.height, buffer).unwrap()
    }
}

#[cfg(feature = "image")]
impl FromRawVideoFrame for image::Rgba32FImage {
    const FORMAT: u32 = aviutl2_sys::common::BI_HF64;

    fn check(_video: &VideoOutputInfo) -> Result<(), String> {
        Ok(())
    }
    unsafe fn from_raw(
        video: &VideoOutputInfo,
        frame_data_ptr: *const u8,
        _last_frame_id: Arc<AtomicUsize>,
        _frame_id: usize,
    ) -> Self {
        let frame_data_ptr = frame_data_ptr as *const f16;
        let buffer = unsafe {
            std::slice::from_raw_parts(frame_data_ptr, (video.width * video.height * 4) as usize)
                .iter()
                .map(|&v| v.to_f32())
                .collect::<Vec<_>>()
        };
        image::ImageBuffer::from_raw(video.width, video.height, buffer).unwrap()
    }
}
