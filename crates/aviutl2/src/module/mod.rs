//! # aviutl2-rs / input
//!
//! AviUtl2の入力プラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`InputPlugin`]トレイトを実装した構造体を定義する
//! 2. [`crate::register_input_plugin!`]マクロを使用してプラグインを登録する
//!
//! サンプルは<https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/image-rs-input>を参照してください。

mod binding;
mod param;

pub use super::common::*;
pub use binding::*;
pub use param::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
