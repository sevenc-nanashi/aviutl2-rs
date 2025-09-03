# Rusty Image Input

入力プラグインのサンプルです。
[image-rs](https://github.com/image-rs/image)を使用して画像を読み込みます。
アニメーション画像（APNG、GIFなど）にも対応しています。

## インストール

`C:\ProgramData\aviutl2\Plugin` に `rusty_image.aui2` を配置してください。

## ベンチマーク

ベンチマーク環境：

- プロジェクトファイル：`projects/many_apngs.aup2`
- CPU：i7-1360P
- メモリ：16GB
- 画像の入っているSSD：NVMe（Samsung MZVL2512HDJD-00B07）
- OS：Windows 11 22H2

L-Smash Works：[Build-2025-09-02-02-14-34](https://github.com/Mr-Ojii/L-SMASH-Works-Auto-Builds/releases/tag/build-2025-09-02-02-14-34)、L-SMASH-Works_r1238_Mr-Ojii_vimeo.zip を使用。

<details>
<summary>ベンチマークの計測方法</summary>

- プロジェクトファイルは `projects/many_apngs.aup2` を使用
- 「0フレーム目（読み込み時間）」：VERBOSEログを有効化し、`rendered video frame [0] X ms` のログで計測
- 「読み込み後のメモリ使用量」：0フレーム目のレンダリングが完了した直後に、AviUtlのプロセスのメモリ使用量をタスクマネージャーで確認
- 「レンダリング時間（最小 / 平均 / 最大）」：Rusty Statistics Outputプラグインで計測

</details>

| プラグイン        | 0フレーム目（読み込み時間） | 読み込み後のメモリ使用量 | レンダリング時間（最小 / 平均 / 最大） |
| ----------------- | --------------------------- | ------------------------ | -------------------------------------- |
| L-Smash Works     | 84575ms                     | 826.9MB                  | 160ms / 601ms / 1610ms                 |
| Rusty Image Input | 31088ms                     | 485.0MB                  | 122ms / 596ms / 1387ms                 |
