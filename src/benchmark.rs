use crate::config::Config;
use crate::rendering::{benchmark_render, rearrange_to_aspect};
use crate::text;
use rusttype::Font;
use std::time::Duration;

pub struct BenchmarkResult {
    pub final_image: image::RgbaImage,
    pub letter_count: u32,
    pub duration: Duration,
}

pub fn run(config: &Config, font: &Font) -> BenchmarkResult {
    // Config 経由でフォントサイズを渡すように変更
    let text_stamp = text::generate_stamp(
        font,
        &config.stamp_text,
        config.stamp_margin,
        config.font_size,
    );
    let stamp_width = text_stamp.width();
    let stamp_height = text_stamp.height();

    // ネストされた各エフェクトモジュールを使用してエフェクトを初期化
    let effects: Vec<Box<dyn crate::effects::Effect>> = vec![
        Box::new(crate::effects::glow::GlowEffect::default()),
        Box::new(crate::effects::extrusion::ExtrusionEffect),
        Box::new(crate::effects::rotation::RotationEffect),
        Box::new(crate::effects::sparkle::SparkleEffect),
        Box::new(crate::effects::surreal::SurrealEffect),
        Box::new(crate::effects::colorful::ColorfulEffect),
    ];

    let mut letter_count: u32 = 1;
    let mut lower: u32;
    let threshold_duration = std::time::Duration::from_secs_f64(config.threshold_secs);
    loop {
        lower = letter_count;
        let (duration, _) =
            benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp, &effects);
        let canvas_width = letter_count * stamp_width;
        println!(
            "Benchmarking: letter_count = {} | canvas = {}x{} | duration = {:?}",
            letter_count, canvas_width, stamp_height, duration
        );
        let ratio = threshold_duration.as_secs_f64() / duration.as_secs_f64();
        let factor = if ratio < 1.1 { 1.1 } else { ratio };
        letter_count = (letter_count as f64 * factor).ceil() as u32;
        if duration > threshold_duration {
            println!("Threshold exceeded.");
            break;
        }
    }

    let mut upper = letter_count;
    while upper - lower > 1 {
        let mid = (lower + upper) / 2;
        let (duration, _) =
            benchmark_render(mid, stamp_width, stamp_height, &text_stamp, &effects);
        let canvas_width = mid * stamp_width;
        println!(
            "Binary search: letter_count = {} | canvas = {}x{} | duration = {:?}",
            mid, canvas_width, stamp_height, duration
        );
        if duration > threshold_duration {
            upper = mid;
        } else {
            lower = mid;
        }
    }
    letter_count = lower;
    println!("Benchmark result: letter_count = {} | score = {}", letter_count, letter_count);

    let (final_duration, horizontal_canvas) =
        benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp, &effects);

    let final_image = rearrange_to_aspect(
        &horizontal_canvas,
        stamp_width,
        stamp_height,
        letter_count,
        config.final_width,
        config.final_height,
    );

    BenchmarkResult {
        final_image,
        letter_count,
        duration: final_duration,
    }
}
