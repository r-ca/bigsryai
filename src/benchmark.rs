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

const SAFETY_FACTOR: f64 = 1.1;
const MAX_ITERATIONS: u32 = 100;

pub fn run(config: &Config, font: &Font) -> BenchmarkResult {
    let text_stamp = text::generate_stamp(
        font,
        &config.stamp_text,
        config.stamp_margin,
        config.font_size,
    );
    
    let effects = initialize_effects();
    let (stamp_width, stamp_height) = (text_stamp.width(), text_stamp.height());
    let threshold = Duration::from_secs_f64(config.threshold_secs);

    let letter_count = determine_max_letters(
        stamp_width,
        stamp_height,
        &text_stamp,
        &effects,
        threshold,
    );

    let (duration, horizontal_canvas) = benchmark_render(
        letter_count,
        stamp_width,
        stamp_height,
        &text_stamp,
        &effects,
    );

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
        duration,
    }
}

/// エフェクト初期化
fn initialize_effects() -> Vec<Box<dyn crate::effects::Effect>> {
    vec![
        Box::new(crate::effects::glow::GlowEffect::default()),
        Box::new(crate::effects::extrusion::ExtrusionEffect),
        Box::new(crate::effects::rotation::RotationEffect),
        Box::new(crate::effects::sparkle::SparkleEffect),
        Box::new(crate::effects::surreal::SurrealEffect),
        Box::new(crate::effects::colorful::ColorfulEffect),
    ]
}

/// 最大文字数決定アルゴリズム
fn determine_max_letters(
    stamp_width: u32,
    stamp_height: u32,
    text_stamp: &image::RgbaImage,
    effects: &[Box<dyn crate::effects::Effect>],
    threshold: Duration,
) -> u32 {
    let mut lower_bound = exponential_search(
        stamp_width,
        stamp_height,
        text_stamp,
        effects,
        threshold,
    );

    binary_search(
        lower_bound,
        stamp_width,
        stamp_height,
        text_stamp,
        effects,
        threshold,
    )
}

/// 指数関数的探索フェーズ
fn exponential_search(
    stamp_width: u32,
    stamp_height: u32,
    text_stamp: &image::RgbaImage,
    effects: &[Box<dyn crate::effects::Effect>],
    threshold: Duration,
) -> u32 {
    let mut letter_count = 1;
    for _ in 0..MAX_ITERATIONS {
        let (duration, _) = benchmark_render(
            letter_count,
            stamp_width,
            stamp_height,
            text_stamp,
            effects,
        );

        log_benchmark_step(letter_count, stamp_width, stamp_height, duration);

        if duration > threshold {
            return letter_count;
        }

        letter_count = calculate_next_letter_count(letter_count, duration, threshold);
    }
    letter_count
}

/// 二分探索フェーズ
fn binary_search(
    mut low: u32,
    stamp_width: u32,
    stamp_height: u32,
    text_stamp: &image::RgbaImage,
    effects: &[Box<dyn crate::effects::Effect>],
    threshold: Duration,
) -> u32 {
    let mut high = low * 2;
    let mut best = low;

    for _ in 0..MAX_ITERATIONS {
        if high - low <= 1 {
            break;
        }

        let mid = (low + high) / 2;
        let (duration, _) = benchmark_render(
            mid,
            stamp_width,
            stamp_height,
            text_stamp,
            effects,
        );

        log_benchmark_step(mid, stamp_width, stamp_height, duration);

        if duration <= threshold {
            low = mid;
            best = mid;
        } else {
            high = mid;
        }
    }
    best
}

/// ベンチマークステップのロギング
fn log_benchmark_step(letter_count: u32, width: u32, height: u32, duration: Duration) {
    println!(
        "Benchmark step: letters = {}, canvas = {}x{}, duration = {:?}",
        letter_count,
        letter_count * width,
        height,
        duration
    );
}

/// 次の文字数計算
fn calculate_next_letter_count(current: u32, duration: Duration, threshold: Duration) -> u32 {
    let ratio = threshold.as_secs_f64() / duration.as_secs_f64();
    let factor = if ratio < SAFETY_FACTOR { SAFETY_FACTOR } else { ratio };
    (current as f64 * factor).ceil().max(current as f64 + 1.0) as u32
}
