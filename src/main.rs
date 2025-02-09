mod benchmark;
mod config;
mod rendering;
mod text;
mod effects; // effects 内に glow, extrusion, rotation, sparkle, surreal, colorful などのサブモジュールあり

use rusttype::Font;

fn main() {
    // Config の値はコード内に集めた固定値から作成
    let config = config::Config::new();
    println!("Using threshold: {:.2} seconds", config.threshold_secs);

    // フォントデータ（Nyashi.ttf をプロジェクト内に配置）
    let font_data = include_bytes!("Nyashi.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("Failed to load font");

    let result = benchmark::run(&config, &font);

    result.final_image
        .save(&config.output_file)
        .expect("Failed to save image");

    println!(
        "Benchmark result: letter_count = {} | score = {}",
        result.letter_count, result.letter_count
    );
    println!("Output image saved as {}", config.output_file);
}
