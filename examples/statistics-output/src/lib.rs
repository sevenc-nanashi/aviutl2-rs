use aviutl2::output::OutputPlugin;
use base64::{Engine, engine::general_purpose::STANDARD as base64};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderData {
    version: String,
    width: u32,
    height: u32,
    ms_per_frame: Vec<f64>,
    num_frames: u32,
    total_ms: f64,
    fps: f64,
    start_time: String,
    end_time: String,
}

struct StatisticsPlugin {}

impl OutputPlugin for StatisticsPlugin {
    fn new() -> Self {
        StatisticsPlugin {}
    }

    fn plugin_info(&self) -> aviutl2::output::OutputPluginTable {
        aviutl2::output::OutputPluginTable {
            name: "Rusty Statistics Output Plugin".to_string(),
            information: format!(
                "Statistics Output Plugin for AviUtl / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/statistics-output",
                version = env!("CARGO_PKG_VERSION")
            ),
            output_type: aviutl2::output::OutputType::Video,
            file_filters: vec![aviutl2::FileFilter {
                name: "Statistics Page".to_string(),
                extensions: vec!["html".to_string()],
            }],
            can_config: false,
        }
    }

    fn output(&self, info: aviutl2::output::OutputInfo) -> aviutl2::AnyResult<()> {
        let Some(video_info) = &info.video else {
            return Err(anyhow::anyhow!("No video information available"));
        };
        // バッファを0に設定
        info.set_buffer_size(0, 0);
        let start_time = chrono::Local::now();

        let mut elapsed = Vec::with_capacity(video_info.num_frames as usize);
        let mut time_before = std::time::Instant::now();

        for (_i, _frame) in info.get_video_frames_iter::<aviutl2::output::RawBgrVideoFrame>() {
            let time_after = std::time::Instant::now();
            elapsed.push(time_after.duration_since(time_before).as_secs_f64() * 1000.0);
            time_before = time_after;
        }
        let end_time = chrono::Local::now();

        let total_ms = elapsed.iter().sum::<f64>() * 1000.0;
        let fps = (*video_info.fps.denom() as f64) / (*video_info.fps.numer() as f64);
        let render_data = RenderData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            ms_per_frame: elapsed,
            num_frames: video_info.num_frames,
            total_ms,
            fps,
            start_time: start_time.to_rfc3339(),
            end_time: end_time.to_rfc3339(),
            width: video_info.width,
            height: video_info.height,
        };
        static TEMPLATE: &str = include_str!("../page/dist/index.html");
        let mut page = TEMPLATE.to_string();
        page = page.replace(
            "data-render-data=\"!PLACEHOLDER!\"",
            &format!(
                "data-render-data=\"{}\"",
                base64.encode(serde_json::to_string(&render_data).unwrap())
            ),
        );
        std::fs::write(info.path, page)
            .map_err(|e| anyhow::anyhow!("Failed to write output file: {}", e))?;

        Ok(())
    }
}

aviutl2::register_output_plugin!(StatisticsPlugin);
