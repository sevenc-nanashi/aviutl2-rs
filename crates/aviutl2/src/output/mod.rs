//! # aviutl2-rs / output
//!
//! AviUtl2の出力プラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`OutputPlugin`]トレイトを実装した構造体を定義する
//! 2. [`crate::register_output_plugin!`]マクロを使用してプラグインを登録する
//!
//! サンプルは<https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-output>を参照してください。

mod binding;
mod video_frame;

pub use super::common::*;
pub use binding::*;
pub use video_frame::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
