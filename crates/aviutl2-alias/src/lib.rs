//! # aviutl2-alias
//!
//! AviUtl2のプロジェクトファイル（`*.aup2`）とエイリアスファイル（`*.object`、`*.effect`）で使われている
//! データ構造を読み書きするクレート。
mod table;
mod track;
mod value;

pub use table::*;
pub use track::*;
pub use value::*;
