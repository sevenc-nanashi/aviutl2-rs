use crate::{DEFAULT_ARGS, config::PixelFormat};

#[derive(Debug, Clone)]
pub struct FfmpegPreset {
    #[allow(dead_code)]
    pub id: &'static str, // Unique identifier for the preset
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [&'static str],
    pub pixel_format: PixelFormat,
}

pub static PRESETS: &[&FfmpegPreset] = &[
    &DEFAULT_PRESET,
    &FINAL_MP4_PRESET,
    &YOUTUBE_PRESET,
    &NICONICO_STANDARD_PRESET,
    &NICONICO_MAX_PRESET,
    &PRORES_PRESET,
    &TRANSPARENT_MOV_PRESET,
];

pub static DEFAULT_PRESET: FfmpegPreset = FfmpegPreset {
    id: "default",
    name: "デフォルト",
    description: "デフォルトの最小限のFFmpeg設定。",
    args: DEFAULT_ARGS,
    pixel_format: PixelFormat::Bgr24,
};

pub static FINAL_MP4_PRESET: FfmpegPreset = FfmpegPreset {
    id: "final_mp4",
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
        "-pix_fmt",
        "yuv420p",
        "-b:a",
        "192k",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Bgr24,
};

pub static YOUTUBE_PRESET: FfmpegPreset = FfmpegPreset {
    id: "youtube",
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

pub static NICONICO_STANDARD_PRESET: FfmpegPreset = FfmpegPreset {
    id: "niconico_standard",
    name: "ニコニコ（推奨）",
    description: "ニコニコ動画の推奨設定（H.264 + AAC）。",
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
        "-profile:v",
        "high",
        "-r",
        "{video_fps}",
        "-vbf",
        "2",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-pix_fmt",
        "yuv420p",
        "-ar",
        "{audio_sample_rate}",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Bgr24,
};

pub static NICONICO_MAX_PRESET: FfmpegPreset = FfmpegPreset {
    id: "niconico_max",
    name: "ニコニコ（最高音質）",
    description: "ニコニコ動画の最高音質設定（H.264 + flac）。mkvで出力してください。",
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
        "-crf",
        "18",
        "-pix_fmt",
        "yuv420p",
        "-preset",
        "veryslow",
        "-c:a",
        "flac",
        "{output_path}",
    ],
    pixel_format: PixelFormat::Bgr24,
};

pub static PRORES_PRESET: FfmpegPreset = FfmpegPreset {
    id: "prores",
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
    id: "transparent_mov",
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
    use std::io::Write;

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

    #[test]
    fn test_presets_valid() {
        let width = 64;
        let height = 64;
        let duration = 1.0; // 1 second
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_result");
        let test_image_path = root.join("test_video.raw");
        {
            let mut image = std::fs::File::create(&test_image_path).unwrap();
            for n in 0..(30.0 * duration) as usize {
                for y in 0..height {
                    for x in 0..width {
                        let r = (x * 255 / width) as u8;
                        let g = (y * 255 / height) as u8;
                        let b = (n as f64 / (30.0 * duration) * 255.0) as u8;
                        image.write_all(&[b, g, r]).unwrap(); // BGR format
                    }
                }
            }
        }
        let test_audio_path = root.join("test_audio.raw");
        {
            let mut audio = std::fs::File::create(&test_audio_path).unwrap();
            let freq = 440.0; // A4 note
            for n in 0..(48000.0 * duration) as usize {
                let sample = (n as f64 * freq * 2.0 * std::f64::consts::PI / 48000.0).sin();
                let sample = (sample * 32767.0) as f32; // Convert to f32
                // Write as little-endian f32 (Stereo, 2 channels)
                audio.write_all(&sample.to_le_bytes()).unwrap();
                audio.write_all(&sample.to_le_bytes()).unwrap();
            }
        }
        let video_size = format!("{width}x{height}");
        let base_replacements = vec![
            ("{video_pixel_format}", "bgr24"),
            ("{video_size}", &video_size),
            ("{video_fps}", "30"),
            ("{video_source}", test_image_path.to_str().unwrap()),
            ("{audio_source}", test_audio_path.to_str().unwrap()),
            ("{audio_sample_rate}", "48000"),
            ("{maybe_vflip}", "null"), // No vertical flip for this test
        ];
        for preset in PRESETS {
            let mut replacements: Vec<(&str, &str)> = base_replacements.clone();
            let extension = match preset.id {
                "prores" | "transparent_mov" => "mov",
                "niconico_max" => "mkv",
                _ => "mp4",
            };
            let output_path = root.join(format!("{}_output.{}", preset.id, extension));
            replacements.push(("{output_path}", output_path.to_str().unwrap()));
            let args: Vec<String> = preset
                .args
                .iter()
                .map(|arg| {
                    replacements
                        .iter()
                        .fold(arg.to_string(), |acc, (key, value)| acc.replace(key, value))
                })
                .collect();

            let cmd = std::process::Command::new("ffmpeg")
                .args(&args)
                .output()
                .expect("Failed to execute ffmpeg command");
            if !cmd.status.success() {
                panic!(
                    "Preset '{}' failed with status: {}. Output: {}",
                    preset.id,
                    cmd.status,
                    String::from_utf8_lossy(&cmd.stderr)
                );
            }
        }
    }
}
