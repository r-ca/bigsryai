use image::{imageops, GenericImageView, Rgba, RgbaImage};
use rusttype::Font;
use std::{env, time::Duration};
use sysinfo::System;

fn main() {
    // ----- ベンチマーク部（横一列レンダリング） -----
    let args: Vec<String> = env::args().collect();
    let threshold_secs: f64 = if args.len() > 1 {
        args[1].parse().unwrap_or(2.0)
    } else {
        2.0
    };
    let threshold = Duration::from_secs_f64(threshold_secs);
    println!("使用する閾値:\t{threshold_secs:.2} 秒");

    let mut sys = System::new_all();
    sys.refresh_all();
    println!("■ システムスペック");
    println!(" Total Memory:\t{} MB", sys.total_memory() / 1024);
    println!("---------------------------------------");

    let font_data = include_bytes!("Nyashi.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("フォント読み込み失敗");

    let text_stamp = bigsryai::generate_stamp(&font, "nexryai", 2);
    let stamp_width = text_stamp.width();
    let stamp_height = text_stamp.height();

    // 前回の消費時間から柔軟にジャンプ倍率を算出して letter_count を増加
    let mut letter_count: u32 = 1;
    let mut lower;
    loop {
        lower = letter_count;
        let (duration, _) =
            bigsryai::benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp);
        let canvas_width = letter_count * stamp_width;
        print!(
            "ベンチマーク中:\tねく数 = {letter_count}\tキャンバスサイズ = {canvas_width}x{stamp_height}\t経過時間 = {duration:.2?}"
        );
        let ratio = threshold.as_secs_f64() / duration.as_secs_f64();
        let factor = if ratio < 1.1 { 1.1 } else { ratio };
        letter_count = (f64::from(letter_count) * factor).ceil() as u32;
        if duration <= threshold {
            println!();
        } else {
            println!("<-- 閾値超過");
            break;
        }
    }
    let mut upper = letter_count;

    // 二分探索で最適な letter_count を求める（upper - lower > 1 で終了）
    while upper - lower > 1 {
        let mid = (lower + upper) / 2;
        let (duration, _) = bigsryai::benchmark_render(mid, stamp_width, stamp_height, &text_stamp);
        let canvas_width = mid * stamp_width;
        println!(
            "二分探索中:\tねく数 = {mid}\tキャンバスサイズ = {canvas_width}x{stamp_height}\t経過時間 = {duration:.2?}");
        if duration > threshold {
            upper = mid;
        } else {
            lower = mid;
        }
    }
    letter_count = lower;

    println!("■ ベンチマーク結果");
    println!("ねく数:\t{letter_count}");
    println!("スコア:\t{letter_count}");

    // ----- 最終結果画像生成 -----
    // 並列処理で各セルをレンダリングし、横一列画像 (final_bench_canvas) を生成
    let (_elapsed, final_bench_canvas) =
        bigsryai::benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp);
    // セルを横幅 1920px に合わせ、全体が FHD (1920×1080) になるよう折り返して配置
    let final_canvas_width: u32 = 1920;
    let cells_per_row = if stamp_width == 0 {
        1
    } else {
        final_canvas_width / stamp_width
    };
    let cells_per_row = if cells_per_row == 0 { 1 } else { cells_per_row };
    let rows = letter_count.div_ceil(cells_per_row);
    let natural_width = cells_per_row * stamp_width;
    let natural_height = rows * stamp_height;
    let mut natural_img =
        RgbaImage::from_pixel(natural_width, natural_height, Rgba([255, 255, 255, 255]));
    for i in 0..letter_count {
        let src_x = i * stamp_width;
        let cell = final_bench_canvas
            .view(src_x, 0, stamp_width, stamp_height)
            .to_image();
        let dest_col = i % cells_per_row;
        let dest_row = i / cells_per_row;
        let dest_x = dest_col * stamp_width;
        let dest_y = dest_row * stamp_height;
        imageops::overlay(
            &mut natural_img,
            &cell,
            i64::from(dest_x),
            i64::from(dest_y),
        );
        if i % 100 == 0 || i == letter_count - 1 {
            println!(
                "セル配置中:\t{} / {} \t(自然画像サイズ: {}x{})",
                i + 1,
                letter_count,
                natural_width,
                natural_height
            );
        }
    }
    let final_img = imageops::resize(&natural_img, 1920, 1080, imageops::Lanczos3);

    // 結果テキストのオーバーレイ（右下）
    let text_stamp_result = bigsryai::generate_overlay(
        &font,
        &format!("ねく数: {letter_count}\nスコア: {letter_count}"),
    );
    let overlay_x = final_img
        .width()
        .saturating_sub(text_stamp_result.width() + 10);
    let overlay_y = final_img
        .height()
        .saturating_sub(text_stamp_result.height() + 10);
    let mut final_img = final_img;
    for (px, py, &p) in text_stamp_result.enumerate_pixels() {
        let dest_x = overlay_x + px;
        let dest_y = overlay_y + py;
        if dest_x < final_img.width() && dest_y < final_img.height() {
            final_img.put_pixel(dest_x, dest_y, p);
        }
    }
    final_img
        .save("output.png")
        .expect("結果の書き込みに失敗しました");
    println!("FHDリザルト画像（1920×1080）として output.png に保存しました。");
}
