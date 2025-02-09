use bigsryai::benchmark::run_benchmark;
use rusttype::Font;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let threshold_secs: f64 = if args.len() > 1 {
        args[1].parse().unwrap_or(2.0)
    } else {
        2.0
    };
    println!("Using threshold: {:.2} seconds", threshold_secs);

    // フォントデータ（Nyashi.ttf をプロジェクト内に配置）
    let font_data = include_bytes!("Nyashi.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("Failed to load font");

    run_benchmark(&font);
}
