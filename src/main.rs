use image::{Rgba, RgbaImage, imageops, GenericImageView};
use rusttype::{point, Font, Scale};
use std::env;
use std::time::{Duration, Instant};
use sysinfo::System;

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0.0, 0.0, 0.0),
    };
    ((r * 255.0).round() as u8,
     (g * 255.0).round() as u8,
     (b * 255.0).round() as u8)
}

fn draw_cell(
    canvas: &mut RgbaImage,
    base_x: u32,
    base_y: u32,
    text_stamp: &RgbaImage,
    cell_index: u32,
    stamp_w: u32,
    stamp_h: u32,
) {
    // (1) Glow効果
    let glow_range: i32 = 3;
    for dx in -glow_range..=glow_range {
        for dy in -glow_range..=glow_range {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            let alpha_factor = ((glow_range as f32 - dist) / glow_range as f32).max(0.0) * 0.3;
            for (px, py, &p) in text_stamp.enumerate_pixels() {
                let dest_x = base_x as i32 + dx + px as i32;
                let dest_y = base_y as i32 + dy + py as i32;
                if dest_x >= 0 && dest_y >= 0 &&
                   dest_x < canvas.width() as i32 && dest_y < canvas.height() as i32 {
                    let Rgba([r, g, b, a]) = p;
                    let white = 255u8;
                    let new_r = ((r as f32 * (1.0 - alpha_factor)) + (white as f32 * alpha_factor))
                        .min(255.0) as u8;
                    let new_g = ((g as f32 * (1.0 - alpha_factor)) + (white as f32 * alpha_factor))
                        .min(255.0) as u8;
                    let new_b = ((b as f32 * (1.0 - alpha_factor)) + (white as f32 * alpha_factor))
                        .min(255.0) as u8;
                    canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }

    // (2) Extrusion効果（縦方向の伸びを抑えるため、base_dy を 2 に変更）
    let extrude_steps = 8;
    let base_dx = 5;
    let base_dy = 1;
    for extrude in 0..extrude_steps {
        let off_x = base_x + extrude * base_dx;
        let off_y = base_y + extrude * base_dy;
        let dark_factor = 1.0 - (extrude as f32) / ((extrude_steps + 1) as f32);
        for (px, py, &p) in text_stamp.enumerate_pixels() {
            let distortion_x = 5.0 * (((px as f32) + cell_index as f32) * 0.17).sin();
            let distortion_y = 5.0 * (((py as f32) + cell_index as f32) * 0.17).cos();
            let dest_x = off_x + px + distortion_x.round() as u32;
            let dest_y = off_y + py + distortion_y.round() as u32;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                let Rgba([r, g, b, a]) = p;
                let new_r = ((r as f32) * dark_factor).min(255.0) as u8;
                let new_g = ((g as f32) * dark_factor).min(255.0) as u8;
                let new_b = ((b as f32) * dark_factor).min(255.0) as u8;
                canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
            }
        }
    }

    // (3) 回転・変形効果付き前面描画（縦方向の変位を 0.7 倍にして伸びにくく）
    let center_x = stamp_w as f32 / 2.0;
    let center_y = stamp_h as f32 / 2.0;
    let angle = 0.3 * ((cell_index as f32 * 0.7).sin());
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        let dx = px as f32 - center_x;
        let dy = py as f32 - center_y;
        let rdx = dx * cos_a - dy * sin_a;
        let rdy = dx * sin_a + dy * cos_a;
        let new_x = center_x + rdx;
        let new_y = center_y + rdy * 0.7; // 縦方向に圧縮
        let dest_x = base_x + new_x.round() as u32;
        let dest_y = base_y + new_y.round() as u32;
        if dest_x < canvas.width() && dest_y < canvas.height() {
            let Rgba([r, g, b, a]) = p;
            let new_r = r.saturating_add(30);
            canvas.put_pixel(dest_x, dest_y, Rgba([new_r, g, b, a]));
        }
    }

    // (4) Sparkle効果
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        let Rgba([r, g, b, a]) = p;
        let lum = (r as u32 + g as u32 + b as u32) / 3;
        if lum > 200 && ((px + py + cell_index) % 97 == 0) {
            let dest_x = base_x + px;
            let dest_y = base_y + py;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                canvas.put_pixel(dest_x, dest_y, Rgba([255, 255, 255, a]));
            }
        }
    }

    // (5) 既存のシュール効果：特定条件で若干右へずらし、色味を微調整
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        if ((px + py + cell_index) % 101 == 0) {
            let dest_x = base_x + px + 3; // 横方向に 3 ピクセルずらす
            let dest_y = base_y + py;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                let Rgba([r, g, b, a]) = p;
                let new_r = r.saturating_sub(10);
                let new_g = g.saturating_sub(10);
                let new_b = ((b as u16 + 10).min(255)) as u8;
                canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
            }
        }
    }

    // (6) 新たなカラフルな乱れ効果：多くのピクセルを左右・上下ににゃぐにゃぐ動かし、HSV の色相を用いて色を大幅に変調
    for (px, py, &p) in text_stamp.enumerate_pixels() {
        if ((px + py + cell_index) % 7 == 0) {
            // sin/cos による左右上下の変位（にゃぐにゃ）
            let offset_x = (5.0 * ((px as f32 + cell_index as f32) * 0.27).sin()).round() as i32;
            let offset_y = (5.0 * ((py as f32 + cell_index as f32) * 0.27).cos()).round() as i32;
            let dest_x = base_x as i32 + px as i32 + offset_x;
            let dest_y = base_y as i32 + py as i32 + offset_y;
            if dest_x >= 0 && dest_y >= 0 && dest_x < canvas.width() as i32 && dest_y < canvas.height() as i32 {
                // HSV で新たな色相を算出し、オリジナルと半分ずつ合成
                let hue = ((px as f32 / stamp_w as f32) + (py as f32 / stamp_h as f32) + (cell_index as f32 * 0.05)) % 1.0;
                let (r2, g2, b2) = hsv_to_rgb(hue, 0.9, 1.0);
                let Rgba([r, g, b, a]) = p;
                let new_r = ((r as u16 + r2 as u16) / 2) as u8;
                let new_g = ((g as u16 + g2 as u16) / 2) as u8;
                let new_b = ((b as u16 + b2 as u16) / 2) as u8;
                canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
            }
        }
    }
}

fn benchmark_render(letter_count: u32, stamp_w: u32, stamp_h: u32, text_stamp: &RgbaImage) -> Duration {
    let canvas_width = letter_count * stamp_w;
    let canvas_height = stamp_h;
    let mut canvas = RgbaImage::from_pixel(canvas_width, canvas_height, Rgba([255, 255, 255, 255]));
    let start = Instant::now();
    for i in 0..letter_count {
        let base_x = i * stamp_w;
        let base_y = 0;
        draw_cell(&mut canvas, base_x, base_y, text_stamp, i, stamp_w, stamp_h);
    }
    start.elapsed()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ----- ベンチマーク部（横一列に連ねる） -----
    let args: Vec<String> = env::args().collect();
    let threshold_secs: f64 = if args.len() > 1 { args[1].parse().unwrap_or(10.0) } else { 10.0 };
    let threshold = Duration::from_secs_f64(threshold_secs);
    println!("使用する閾値:\t{:.2} 秒", threshold_secs);

    let mut sys = System::new_all();
    sys.refresh_all();
    println!("■ システムスペック");
    println!(" Total Memory:\t{} MB", sys.total_memory() / 1024);
    println!("---------------------------------------");

    // フォント読み込み＆文字スタンプ生成（フォントサイズ 256）
    let font_data = include_bytes!("Nyashi.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("フォント読み込み失敗");
    let text = "nexryai";
    let scale = Scale::uniform(32.0);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();
    let min_x = glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.min.x)).min().unwrap_or(0);
    let min_y = glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.min.y)).min().unwrap_or(0);
    let max_x = glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.x)).max().unwrap_or(0);
    let max_y = glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.y)).max().unwrap_or(0);
    let text_width = (max_x - min_x) as u32;
    let text_height = (max_y - min_y) as u32;
    let margin = 20;
    let stamp_width = text_width + 2 * margin;
    let stamp_height = text_height + 2 * margin;
    let mut text_stamp = RgbaImage::from_pixel(stamp_width, stamp_height, Rgba([255, 255, 255, 255]));
    let offset = point(margin as f32 - min_x as f32, margin as f32 - min_y as f32 + v_metrics.ascent);
    for glyph in font.layout(text, scale, offset) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let x = bb.min.x + gx as i32;
                let y = bb.min.y + gy as i32;
                if x >= 0 && y >= 0 && (x as u32) < stamp_width && (y as u32) < stamp_height {
                    let hue = (x as f32) / (stamp_width as f32);
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    let blended_r = (r as f32 * v + 255.0 * (1.0 - v)).round() as u8;
                    let blended_g = (g as f32 * v + 255.0 * (1.0 - v)).round() as u8;
                    let blended_b = (b as f32 * v + 255.0 * (1.0 - v)).round() as u8;
                    text_stamp.put_pixel(x as u32, y as u32, Rgba([blended_r, blended_g, blended_b, 255]));
                }
            });
        }
    }

    // 前回の消費時間から次回のジャンプ倍率を柔軟に算出
    let mut letter_count: u32 = 1;
    let mut lower = letter_count;
    let mut duration = benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp);
    while duration <= threshold {
        println!("ベンチマーク中:\tねく数 = {}\tキャンバスサイズ = {}x{}\t経過時間 = {:.2?}",
                 letter_count, letter_count * stamp_width, stamp_height, duration);
        lower = letter_count;
        let ratio = threshold.as_secs_f64() / duration.as_secs_f64();
        // ratio が 1.1 未満の場合は最低 1.1 倍とする
        let factor = if ratio < 1.1 { 1.1 } else { ratio };
        letter_count = (letter_count as f64 * factor).ceil() as u32;
        duration = benchmark_render(letter_count, stamp_width, stamp_height, &text_stamp);
    }
    let mut upper = letter_count;
    println!("ベンチマーク中:\tねく数 = {}\tキャンバスサイズ = {}x{}\t経過時間 = {:.2?}  <-- 閾値超過",
             letter_count, letter_count * stamp_width, stamp_height, duration);

    // 二分探索（upper - lower > 1 で終了）
    while upper - lower > 1 {
        let mid = (lower + upper) / 2;
        let duration = benchmark_render(mid, stamp_width, stamp_height, &text_stamp);
        println!("二分探索中:\tねく数 = {}\tキャンバスサイズ = {}x{}\t経過時間 = {:.2?}",
                 mid, mid * stamp_width, stamp_height, duration);
        if duration > threshold {
            upper = mid;
        } else {
            lower = mid;
        }
    }
    letter_count = lower;
    println!("■ ベンチマーク結果");
    println!("ねく数:\t{}", letter_count);
    println!("スコア:\t{}", letter_count);

    // ----- 最終結果画像生成 -----
    // 横一列にレンダリングした画像（final_bench_canvas）を作成
    let bench_width = letter_count * stamp_width;
    let bench_height = stamp_height;
    let mut final_bench_canvas = RgbaImage::from_pixel(bench_width, bench_height, Rgba([255, 255, 255, 255]));
    for i in 0..letter_count {
        let base_x = i * stamp_width;
        let base_y = 0;
        draw_cell(&mut final_bench_canvas, base_x, base_y, &text_stamp, i, stamp_width, stamp_height);
        if i % 100 == 0 || i == letter_count - 1 {
            println!("最終レンダリング中:\tセル {} / {} \t(キャンバスサイズ: {}x{})",
                     i + 1, letter_count, bench_width, bench_height);
        }
    }

    // セルを FHD サイズ（1920×1080）に折り返して配置
    let final_canvas_width: u32 = 1920;
    let cells_per_row = if stamp_width == 0 { 1 } else { final_canvas_width / stamp_width };
    let cells_per_row = if cells_per_row == 0 { 1 } else { cells_per_row };
    let rows = (letter_count + cells_per_row - 1) / cells_per_row;
    let natural_width = cells_per_row * stamp_width;
    let natural_height = rows * stamp_height;
    let mut natural_img = RgbaImage::from_pixel(natural_width, natural_height, Rgba([255, 255, 255, 255]));
    for i in 0..letter_count {
        let src_x = i * stamp_width;
        let cell = final_bench_canvas.view(src_x, 0, stamp_width, stamp_height).to_image();
        let dest_col = i % cells_per_row;
        let dest_row = i / cells_per_row;
        let dest_x = dest_col * stamp_width;
        let dest_y = dest_row * stamp_height;
        imageops::replace(&mut natural_img, &cell, dest_x as i64, dest_y as i64);
        if i % 100 == 0 || i == letter_count - 1 {
            println!("セル配置中:\t{} / {} \t(自然画像サイズ: {}x{})",
                     i + 1, letter_count, natural_width, natural_height);
        }
    }
    let mut final_img = imageops::resize(&natural_img, 1920, 1080, imageops::Lanczos3);

    // 結果テキストのオーバーレイ（右下）
    let result_text = format!("ねく数: {}\nスコア: {}", letter_count, letter_count);
    let text_scale = Scale::uniform(64.0);
    let text_v_metrics = font.v_metrics(text_scale);
    let result_glyphs: Vec<_> = font.layout(&result_text, text_scale, point(0.0, text_v_metrics.ascent)).collect();
    let res_min_x = result_glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.min.x)).min().unwrap_or(0);
    let res_min_y = result_glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.min.y)).min().unwrap_or(0);
    let res_max_x = result_glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.x)).max().unwrap_or(0);
    let res_max_y = result_glyphs.iter().filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.y)).max().unwrap_or(0);
    let res_text_width = (res_max_x - res_min_x) as u32;
    let res_text_height = (res_max_y - res_min_y) as u32;
    let mut text_stamp_result = RgbaImage::from_pixel(res_text_width + 20, res_text_height + 20, Rgba([0, 0, 0, 0]));
    let res_offset = point(10.0 - res_min_x as f32, 10.0 - res_min_y as f32 + text_v_metrics.ascent);
    for glyph in font.layout(&result_text, text_scale, res_offset) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let x = bb.min.x + gx as i32;
                let y = bb.min.y + gy as i32;
                if x >= 0 && y >= 0 &&
                   (x as u32) < text_stamp_result.width() &&
                   (y as u32) < text_stamp_result.height() {
                    let alpha = (v * 255.0).round() as u8;
                    text_stamp_result.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, alpha]));
                }
            });
        }
    }
    let overlay_x = final_img.width().saturating_sub(text_stamp_result.width() + 10);
    let overlay_y = final_img.height().saturating_sub(text_stamp_result.height() + 10);
    for (px, py, &p) in text_stamp_result.enumerate_pixels() {
        let dest_x = overlay_x + px;
        let dest_y = overlay_y + py;
        if dest_x < final_img.width() && dest_y < final_img.height() {
            final_img.put_pixel(dest_x, dest_y, p);
        }
    }
    final_img.save("output.png")?;
    println!("FHDリザルト画像（1920×1080）として output.png に保存しました。");

    Ok(())
}
