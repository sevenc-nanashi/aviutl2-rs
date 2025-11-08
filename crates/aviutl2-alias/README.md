# aviutl2-alias

[![Crates.io](https://img.shields.io/crates/v/aviutl2-alias.svg)](https://crates.io/crates/aviutl2-alias)
[![Documentation](https://docs.rs/aviutl2-alias/badge.svg)](https://docs.rs/aviutl2-alias)

AviUtl2のプロジェクトファイル（`*.aup2`）とエイリアスファイル（`*.object`、`*.effect`）で使われているデータ構造を読み書きするクレート。

## 使い方

```rust
use aviutl2_alias::{Table, TrackItem};

fn main() -> anyhow::Result<()> {
    // `.aup2`ファイルからテーブルを読み込む
    let input = std::fs::read_to_string("project.aup2")?;
    let root: Table = input.parse()?;

    let project = root.get_table("project").expect("project section");
    let version = project.parse_value::<u32>("version").unwrap()?;

    println!("AviUtl2 Project Version: {}", version);
    Ok(())
}
```

## ライセンス

MIT Licenseで公開されています。
