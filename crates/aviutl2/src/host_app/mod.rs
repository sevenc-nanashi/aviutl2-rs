//! # aviutl2-rs / host_app
//!
//! AviUtl2の汎用プラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`HostAppPlugin`]トレイトを実装し、かつ
//!    [`#[aviutl2::plugin(HostAppPlugin)]`][crate::plugin]属性を付与した構造体を定義する
//! 2. [`crate::register_host_app_plugin!`]マクロを使用してプラグインを登録する
//!
//! サンプルは<https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/todo>を参照してください。

mod binding;

pub use super::common::*;
pub use binding::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
