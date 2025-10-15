# Rusty Pixel Sort Filter

フィルタプラグインのサンプルです。
ピクセルソートを実装しています。

## Contributing

せっかくなので爆速なピクセルソートを作りたいです。
一応実験として以下のFeature Flagを用意しています：
- `rayon` - Rayonを使ってマルチスレッド化します。
  - `rayon-sort-rows` - ピクセルソートで行ごとにRayonを使います。
  - `rayon-sort-inner` - ピクセルソートの行内のソートにRayonを使います。
  - `rayon-rotate` - ピクセルの回転にRayonを使います。
- `simd` - SIMDを使って高速化します。
  - `simd-luminance` - 輝度計算にSIMDを使います。
  - `simd-threshold` - ピクセルの閾値計算にSIMDを使います。

挑戦、お待ちしております。

## インストール

`C:\ProgramData\aviutl2\Plugin` に `rusty_pixelsort.auf2` を配置してください。
