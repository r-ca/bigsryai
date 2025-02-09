mod benchmark;
mod config;
mod rendering;
mod text;
mod effects;
mod annotation;

use sysinfo::System;
use rusttype::Font;
use ab_glyph::FontArc;

fn main() {
    let config = config::Config::new();
    println!("Using threshold: {:.2} seconds", config.threshold_secs);

    let font_data = include_bytes!("Nyashi.ttf") as &[u8];
    let rt_font = Font::try_from_bytes(font_data).expect("Failed to load rusttype font");

    let result = benchmark::run(&config, &rt_font);

    let mut sys = System::new_all();
    sys.refresh_all();

    let ab_font = FontArc::try_from_slice(font_data).expect("Failed to load font via ab_glyph");

    let annotated_image = annotation::annotate_image(result.final_image, result.letter_count, result.duration, &ab_font, &sys);

    annotated_image
        .save(&config.output_file)
        .expect("Failed to save image");

    println!(
        "Benchmark result: letter_count = {} | score = {}",
        result.letter_count, result.letter_count
    );
    println!("Output image saved as {}", config.output_file);
}
