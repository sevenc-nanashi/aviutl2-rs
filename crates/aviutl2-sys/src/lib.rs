//! # aviutl2-sys
//!
//! AviUtl2 SDKのヘッダーのRust移植。
//!
//! - [`input2`][]：`input2.h`相当。
//! - [`output2`][]：`output2.h`相当。
//! - [`filter2`][]：`filter2.h`相当。
//! - [`module2`][]：`module2.h`相当。
//!
//! - [`common`][]：共通定義。

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod common;
pub mod filter2;
pub mod input2;
pub mod output2;
pub mod module2;
