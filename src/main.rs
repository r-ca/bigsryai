mod benchmark;
mod config;
mod rendering;
mod text;
mod effects;    // effects 内に各サブモジュール（glow, extrusion, etc.）
mod annotation;

use rusttype::Font;
use sysinfo::System;

fn main() {
    let config = config::Config::new();
    println!("Using threshold: {:.2} seconds", config.threshold_secs);

    // 組み込んだフォント（例: Nyashi.ttf を src/ に配置）
    let font_data = include_bytes!("Nyashi.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("Failed to load font");

    let result = benchmark::run(&config, &font);

    // sysinfo の System インスタンスは 1 度だけ作成して更新する
    let mut sys = System::new_all();
    sys.refresh_all();

    // 結果画像にオーバーレイ・ブラー・テキスト（カウント＋スペック情報）を追加する
    let annotated_image = annotation::annotate_image(result.final_image, result.letter_count, &font, &sys);

    annotated_image
        .save(&config.output_file)
        .expect("Failed to save image");

    println!(
        "Benchmark result: letter_count = {} | score = {}",
        result.letter_count, result.letter_count
    );
    println!("Output image saved as {}", config.output_file);
}
