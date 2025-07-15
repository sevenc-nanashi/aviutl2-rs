# Rusty FFmpeg Output

出力プラグインのサンプルです。

- FFmpegを使用して動画を出力する
- eguiを使用してFFmpegのオプションを設定する

推奨される代替：[rigaya/ffmpegOut](https://github.com/rigaya/ffmpegOut)

## 比較

- プロジェクト：[dvd_for_10_minutes.aup2](../../dvd_for_10_minutes.aup2)
- 計測方法：iPhoneのストップウォッチ + 目視。
    - カウント開始：「上書きしますか？」のダイアログでEnterを押してから
    - カウント終了：ステータスバーが青くなるまで
- 設定：
    - ffmpegOut：デフォルト、出力拡張子：`.mp4`
    - Rusty FFmpeg Output：デフォルト

### 
- CPU：i7-1360P、RAM：16GB、OS：Windows 11 Pro、書き出し先：SoftPerfect RAM Disk（1GB）
    - ffmpegOut：6m14s、48.1fps
    - Rusty FFmpeg Output：4m07s、72.9fps
