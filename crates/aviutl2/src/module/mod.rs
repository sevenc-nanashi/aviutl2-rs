//! # aviutl2-rs / module
//!
//! AviUtl2のスクリプトモジュールを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`ScriptModule`]トレイトを実装し、かつ
//!    [`#[aviutl2::plugin(ScriptModule)]`][crate::plugin]属性を付与した構造体を定義する
//! 2. [`crate::module::functions`]で関数を定義する
//! 3. [`crate::register_script_module!`]マクロを使用してプラグインを登録する
//!
//! サンプルは<https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/username-module>を参照してください。

mod binding;
mod param;

pub use super::common::*;
pub use binding::*;
pub use param::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
