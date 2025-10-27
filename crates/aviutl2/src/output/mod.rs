//! # aviutl2-rs / output
//!
//! AviUtl2の出力プラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`OutputPlugin`]トレイトを実装し、かつ
//!    [`#[aviutl2::plugin(OutputPlugin)]`][crate::plugin]属性を付与した構造体を定義する
//! 2. [`crate::register_output_plugin!`]マクロを使用してプラグインを登録する
//!
//! サンプルは<https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-output>を参照してください。

mod binding;
pub mod video_frame;

pub use super::common::*;
pub use binding::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
