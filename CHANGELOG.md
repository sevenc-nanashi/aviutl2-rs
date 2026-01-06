# 変更履歴

## Unreleased

- **Breaking:** APIからのResultはanyhow::Resultを使わないように変更
- **Breaking:** module: IntoScriptModuleReturnValue traitの返り値をanyhow::Errorから変更
- **Breaking:** module: `&[T]`、`[T]`は`Vec<T>`として返すように変更
- **Breaking:** `odbg!`、`oprintln!`を削除
- **Breaking:** filter: `FilterPluginTable::filter_type`、`as_object`を`FilterPluginTable::flags`に統合
- `log`クレートで本体のログに出力できるように
- aviutl2-aliasを追加
- filter: effect_idを追加

### デモプラグイン

- binaural、equalizer: effect_idに対応

## [0.10.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.10.0)

- **Breaking:** AviUtl2Version::newの引数をu32からu8に変更
- **Breaking:** プラグイン定義に`#[aviutl2::plugin(...)]`属性を必須化
- **Breaking:** VideoFrame系を`aviutl2::output::video_frame`モジュールに移動
- スクリプトモジュールを追加

### デモプラグイン

- ユーザー名を取得するスクリプトモジュールを追加

## [0.9.6](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.6)

- ドキュメントのビルドが通らなかったのを修正

### デモプラグイン

- pixelsort: パフォーマンスを向上

## [0.9.5](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.5)

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
