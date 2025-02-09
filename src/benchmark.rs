use image::RgbaImage;
use crate::rendering::{benchmark_render, rearrange_to_aspect};
use crate::text;
use rusttype::Font;

/// ベンチマーク処理およびスコア算出ロジック
pub fn run_benchmark(font: &Font) {
    // テキストスタンプ画像生成
    let text_stamp = text::generate_stamp(font, "nexryai", 2);
    let stamp_width = text_stamp.width();
    let stamp_height = text_stamp.height();

    // 使用する各種エフェクトを配列で用意
    let effects: Vec<Box<dyn crate::effects::Effect>> = vec![
        Box::new(crate::effects::GlowEffect::default()),
        Box::new(crate::effects::ExtrusionEffect),
        Box::new(crate::effects::RotationEffect),
        Box::new(crate::effects::SparkleEffect),
        Box::new(crate::effects::SurrealEffect),
        Box::new(crate::effects::ColorfulEffect),
    ];

    // 初期の letter_count により負荷を段階的に増加
    let mut letter_count: u32 = 1;
    let mut lower;
    loop {
        lower = letter_count;
        let (duration, _) = benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp, &effects);
        let canvas_width = letter_count * stamp_width;
        println!(
            "Benchmarking: letter_count = {} | canvas = {}x{} | duration = {:?}",
            letter_count, canvas_width, stamp_height, duration
        );
        let threshold = std::time::Duration::from_secs_f64(2.0);
        let ratio = threshold.as_secs_f64() / duration.as_secs_f64();
        let factor = if ratio < 1.1 { 1.1 } else { ratio };
        letter_count = (letter_count as f64 * factor).ceil() as u32;
        if duration > threshold {
            println!("Threshold exceeded.");
            break;
        }
    }
    let mut upper = letter_count;
    while upper - lower > 1 {
        let mid = (lower + upper) / 2;
        let (duration, _) = benchmark_render(mid, stamp_width, stamp_height, &text_stamp, &effects);
        let canvas_width = mid * stamp_width;
        println!(
            "Binary search: letter_count = {} | canvas = {}x{} | duration = {:?}",
            mid, canvas_width, stamp_height, duration
        );
        let threshold = std::time::Duration::from_secs_f64(2.0);
        if duration > threshold {
            upper = mid;
        } else {
            lower = mid;
        }
    }
    letter_count = lower;
    println!("Benchmark result: letter_count = {} | score = {}", letter_count, letter_count);

    // 横一列のキャンバスを生成
    let (_elapsed, horizontal_canvas) =
        benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp, &effects);

    // 横一列キャンバスをグリッド化して最終出力画像に変換（FHD: 1920x1080 の例）
    let final_img = rearrange_to_aspect(&horizontal_canvas, stamp_width, stamp_height, letter_count, 1920, 1080);
    final_img.save("output.png").expect("Failed to save image");
    println!("Output image saved as output.png");
}
