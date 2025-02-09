use image::{Rgba, RgbaImage};
use rusttype::Font;
use rayon::prelude::*;
use std::time::Duration;

use crate::rendering::{benchmark_render, rearrange_by_flat_pixels};
use crate::text;

/// 各文字ごとのベンチマーク結果
struct LetterBenchmarkResult {
    letter: char,
    letter_count: u32,
    score: u32,
    avg_duration: f64, // 平均レンダリング時間（秒）
    horizontal_canvas: RgbaImage,
}

/// 各文字（n,e,x,r,y,a,i）を個別にレンダリングし、
/// 全コアを利用して並列にベンチマークを実施する例
pub fn run_benchmark(font: &Font) {
    // 各文字ごとにテキストスタンプ画像を生成（1文字ずつレンダリング）
    let margin = 2;
    let letters: Vec<char> = "nexryai".chars().collect();
    let stamps: Vec<(char, RgbaImage)> = letters
        .into_iter()
        .map(|c| (c, text::generate_stamp(font, &c.to_string(), margin)))
        .collect();

    // 使用する各種エフェクトは従来通り
    let effects: Vec<Box<dyn crate::effects::Effect>> = vec![
        Box::new(crate::effects::GlowEffect::default()),
        Box::new(crate::effects::ExtrusionEffect),
        Box::new(crate::effects::RotationEffect),
        Box::new(crate::effects::SparkleEffect),
        Box::new(crate::effects::SurrealEffect),
        Box::new(crate::effects::ColorfulEffect),
    ];

    // ベンチマークのしきい値（例：2秒）
    let threshold = Duration::from_secs_f64(2.0);
    // 各文字ごとのベンチマークを全コア並列で実施
    let results: Vec<LetterBenchmarkResult> = stamps
        .into_par_iter()
        .map(|(letter, stamp)| {
            let stamp_width = stamp.width();
            let stamp_height = stamp.height();
            // まずletter_countを段階的に増やし、thresholdを超えるまでレンダリングを試す
            let mut letter_count: u32 = 1;
            let mut lower;
            loop {
                lower = letter_count;
                let (duration, _) = benchmark_render(letter_count, stamp_width, stamp_height, &stamp, &effects);
                println!(
                    "Benchmarking letter '{}': letter_count = {} | canvas = {}x{} | duration = {:?}",
                    letter,
                    letter_count,
                    letter_count * stamp_width,
                    stamp_height,
                    duration
                );
                // 現在のdurationに対して、thresholdとの比率から増加係数を決定
                let ratio = threshold.as_secs_f64() / duration.as_secs_f64();
                let factor = if ratio < 1.1 { 1.1 } else { ratio };
                letter_count = (letter_count as f64 * factor).ceil() as u32;
                if duration > threshold {
                    println!("Letter '{}' threshold exceeded.", letter);
                    break;
                }
            }
            // 二分探索で最適なletter_countを求める
            let mut upper = letter_count;
            while upper - lower > 1 {
                let mid = (lower + upper) / 2;
                let (duration, _) =
                    benchmark_render(mid, stamp_width, stamp_height, &stamp, &effects);
                println!(
                    "Binary search letter '{}': letter_count = {} | canvas = {}x{} | duration = {:?}",
                    letter,
                    mid,
                    mid * stamp_width,
                    stamp_height,
                    duration
                );
                if duration > threshold {
                    upper = mid;
                } else {
                    lower = mid;
                }
            }
            letter_count = lower;
            println!(
                "Benchmark result for letter '{}': letter_count = {} | score = {}",
                letter, letter_count, letter_count
            );

            // 安定性の指標として、複数回計測して平均レンダリング時間を算出（例：5回）
            let iterations = 5;
            let mut total_duration = 0.0;
            for _ in 0..iterations {
                let (duration, _) =
                    benchmark_render(letter_count, stamp_width, stamp_height, &stamp, &effects);
                total_duration += duration.as_secs_f64();
            }
            let avg_duration = total_duration / iterations as f64;
            println!(
                "Average duration for letter '{}': {:.3} sec",
                letter, avg_duration
            );

            // 最終的な水平キャンバスを生成
            let (_elapsed, horizontal_canvas) =
                benchmark_render(letter_count, stamp_width, stamp_height, &stamp, &effects);

            LetterBenchmarkResult {
                letter,
                letter_count,
                score: letter_count,
                avg_duration,
                horizontal_canvas,
            }
        })
        .collect();

    // 複数の文字の水平キャンバスを縦に連結して最終画像を作成
    let max_width = results
        .iter()
        .map(|r| r.horizontal_canvas.width())
        .max()
        .unwrap_or(0);
    let total_height: u32 = results.iter().map(|r| r.horizontal_canvas.height()).sum();
    let mut final_img =
        RgbaImage::from_pixel(max_width, total_height, Rgba([255, 255, 255, 255]));
    let mut current_y = 0;
    for res in results.iter() {
        let canvas = &res.horizontal_canvas;
        for y in 0..canvas.height() {
            for x in 0..canvas.width() {
                let pixel = canvas.get_pixel(x, y);
                final_img.put_pixel(x, current_y + y, *pixel);
            }
        }
        current_y += canvas.height();
    }

    // ※必要に応じて、ここで rearrange_by_flat_pixels を用いて再配置することも可能
    final_img
        .save("output.png")
        .expect("Failed to save image");
    println!("Output image saved as output.png");

    // 各文字のベンチマーク結果を Markdown 表として出力
    println!("Benchmark Summary:");
    println!("|Letter|Letter Count|Avg Duration (sec)|Score|");
    println!("|---|---|---|---|");
    for res in results.iter() {
        println!(
            "|{}|{}|{:.3}|{}|",
            res.letter, res.letter_count, res.avg_duration, res.score
        );
    }
}
