use criterion::{Criterion, criterion_group, criterion_main};
use rusty_pixelsort_filter::{FilterConfig, pixelsort};

pub fn criterion_benchmark(c: &mut Criterion) {
    let images = std::fs::read_dir("benches/assets").unwrap();
    for entry in images {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jpeg") {
            continue;
        }
        let img = image::open(&path).unwrap().to_rgba8();
        let img = img
            .pixels()
            .map(|p| aviutl2::filter::RgbaPixel {
                r: p[0],
                g: p[1],
                b: p[2],
                a: p[3],
            })
            .collect::<Vec<_>>();
        c.bench_function(
            &format!("pixelsort {}", path.file_name().unwrap().to_str().unwrap()),
            |b| {
                b.iter(|| {
                    let img = img.clone();
                    pixelsort(
                        &FilterConfig::default(),
                        std::hint::black_box(img),
                        100,
                        100,
                    );
                })
            },
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
