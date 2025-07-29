use crate::{DEFAULT_ARGS, config::PixelFormat};

#[derive(Debug, Clone)]
pub struct FfmpegPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [&'static str],
    pub pixel_format: PixelFormat,
}

pub static PRESETS: &[&FfmpegPreset] = &[
    &DEFAULT_PRESET,
    &FINAL_MP4_PRESET,
    &YOUTUBE_PRESET,
    &PRORES_PRESET,
    &TRANSPARENT_MOV_PRESET,
];

pub static DEFAULT_PRESET: FfmpegPreset = FfmpegPreset {
    name: "デフォルト",
    description: "デフォルトの最小限のFFmpeg設定。",
    args: DEFAULT_ARGS,
    pixel_format: PixelFormat::Bgr24,
};

pub static FINAL_MP4_PRESET: FfmpegPreset = FfmpegPreset {
    name: "Final MP4",
    description: "高品質なH.264/AAC形式で出力。配布・投稿に最適。",
    args: &[
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "{video_pixel_format}",
        "-video_size",
        "{video_size}",
        "-framerate",
        "{video_fps}",
        "-i",
        "{video_source}",
        "-f",
        "f32le",
        "-ar",
        "{audio_sample_rate}",
        "-ac",
        "2",
        "-i",
        "{audio_source}",
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-vf",
        "{maybe_vflip}",
        "-c:v",
        "libx264",
        "-preset",
        "slow",
        "-crf",
        "18",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Bgr24,
};

pub static YOUTUBE_PRESET: FfmpegPreset = FfmpegPreset {
    name: "YouTube",
    description: "YouTube投稿向けの推奨設定（H.264 + AAC + faststart）。",
    args: &[
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "{video_pixel_format}",
        "-video_size",
        "{video_size}",
        "-framerate",
        "{video_fps}",
        "-i",
        "{video_source}",
        "-f",
        "f32le",
        "-ar",
        "{audio_sample_rate}",
        "-ac",
        "2",
        "-i",
        "{audio_source}",
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-vf",
        "{maybe_vflip}",
        "-c:v",
        "libx264",
        "-preset",
        "slow",
        "-crf",
        "18",
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Bgr24,
};

pub static PRORES_PRESET: FfmpegPreset = FfmpegPreset {
    name: "ProRes HQ",
    description: "中間編集用の高品質ProRes 422 HQ（Apple互換）。",
    args: &[
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "{video_pixel_format}",
        "-video_size",
        "{video_size}",
        "-framerate",
        "{video_fps}",
        "-i",
        "{video_source}",
        "-f",
        "f32le",
        "-ar",
        "{audio_sample_rate}",
        "-ac",
        "2",
        "-i",
        "{audio_source}",
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-vf",
        "{maybe_vflip}",
        "-c:v",
        "prores_ks",
        "-profile:v",
        "3",
        "-c:a",
        "pcm_s16le",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Pa64,
};

pub static TRANSPARENT_MOV_PRESET: FfmpegPreset = FfmpegPreset {
    name: "透過mov",
    description: "アルファチャンネル付きProRes 4444でMOV出力（透過対応の高品質動画）。",
    args: &[
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "{video_pixel_format}",
        "-video_size",
        "{video_size}",
        "-framerate",
        "{video_fps}",
        "-i",
        "{video_source}",
        "-f",
        "f32le",
        "-ar",
        "{audio_sample_rate}",
        "-ac",
        "2",
        "-i",
        "{audio_source}",
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-vf",
        "{maybe_vflip}",
        "-c:v",
        "prores_ks",
        "-profile:v",
        "4", // ProRes 4444
        "-pix_fmt",
        "yuva444p10le", // preserve alpha
        "-c:a",
        "pcm_s16le",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Pa64,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::REQUIRED_ARGS;

    #[test]
    fn test_presets() {
        assert!(!PRESETS.is_empty());
        for preset in PRESETS {
            assert!(!preset.name.is_empty());
            assert!(!preset.description.is_empty());
            assert!(!preset.args.is_empty());
            assert!(REQUIRED_ARGS.iter().all(|arg| preset.args.contains(arg)));
        }
    }
}
