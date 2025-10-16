# 変更履歴

## Unreleased

（なし）

### デモプラグイン

- binaural: 今度こそクラッシュを修正

## [0.9.4](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.4)

（なし）

### デモプラグイン

- binaural: クラッシュを修正

## [0.9.2](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.2)

- 公開手順を修正

### デモプラグイン

（なし）

## <s>[0.9.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.1)</s>

- **Breaking:** MSRVを1.90.0に
- crates.ioへの公開を同時に行うようにする

### デモプラグイン

（なし）

## <s>[0.9.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.0)</s>

- **Breaking:** AviUtl2の最小サポートをbeta14に引き上げ
- **Breaking:** aviutl2-sysの共通定義をaviutl2_sys::commonに移動
- **Breaking:** `InputPlugin::new`、`OutputPlugin::new`の引数に`AvUtl2Info`を追加

- 変更履歴を追加
- フィルタプラグインを追加
- `aviutl2-macros`クレートを追加
- `aviutl2::file_filters!`マクロを追加

### デモプラグイン

- **Breaking:** 名前を`Rusty [...] (Input|Output|Filter)`に変更
