use image::{Rgba, RgbaImage};
use image::imageops::{self, overlay};
use imageproc::drawing::{draw_text_mut, text_size, draw_filled_rect_mut};
use imageproc::rect::Rect;
use rusttype::{Font, Scale, point};
use sysinfo::System;
use ab_glyph::{FontArc, PxScale};

pub fn annotate_image(mut img: RgbaImage, count: u32, font: &FontArc, sys: &System) -> RgbaImage {
    // --- オーバーレイ領域の設定 ---
    let overlay_margin_x = img.width() / 10;
    let overlay_margin_y = img.height() / 10;
    let overlay_x = overlay_margin_x;
    let overlay_y = overlay_margin_y;
    let overlay_w = img.width() - 2 * overlay_margin_x;
    let overlay_h = img.height() - 2 * overlay_margin_y;

    // --- オーバーレイ背景の作成 ---
    // 半透明の白（アルファ値128）で埋めた画像を作成し、元画像に重ねる
    let overlay_color = Rgba([255, 255, 255, 128]);
    let overlay_img = RgbaImage::from_pixel(overlay_w, overlay_h, overlay_color);
    overlay(&mut img, &overlay_img, overlay_x.into(), overlay_y.into());


    // --- sysinfo からシステム情報を取得 ---
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let os_info = sysinfo::System::long_os_version().unwrap_or_else(|| "Unknown OS".to_string());
    let _cpu_brand_name = sys.cpus().get(0).map(|c| c.brand());
    let _cpu_name = sys.cpus().get(0).map(|c| c.name());
    let cpu_name = match (_cpu_brand_name, _cpu_name) {
        (Some(brand), Some(name)) => format!("{} ({})", brand, name),
        (Some(brand), None) => brand.to_string(),
        (None, Some(name)) => name.to_string(),
        _ => "Unknown CPU".to_string(),
    };
    let core_count = sys.cpus().len();
    let total_memory_kb = sys.total_memory();
    let used_memory_kb = sys.used_memory();

    let total_memory_mb = total_memory_kb as f64 / 1024.0;
    let used_memory_mb = used_memory_kb as f64 / 1024.0;
    let total_memory_gb = total_memory_mb / 1024.0;
    let used_memory_gb = used_memory_mb / 1024.0;

    // --- 表示するテキストの作成 ---
    let count_text = count.to_string();
    let specs_lines = vec![
        format!("Hostname: {}", hostname),
        format!("CPU: {} ({} cores)", cpu_name, core_count),
        format!("OS: {}", os_info),
        format!(
            "Memory: {:.2} GB ({:.0} MB) / {:.2} GB ({:.0} MB)",
            used_memory_gb, used_memory_mb, total_memory_gb, total_memory_mb
        ),
    ];

    // --- テキスト描画用フォントサイズの設定 ---
    let count_font_size = overlay_h as f32 * 0.2;
    let specs_font_size = overlay_h as f32 * 0.05;
    let count_scale = PxScale { x: count_font_size, y: count_font_size };
    let specs_scale = PxScale { x: specs_font_size, y: specs_font_size };

    // --- テキスト描画位置（オーバーレイ内での余白も相対値で設定） ---
    let inner_margin = overlay_w as f32 * 0.05;
    let (count_width, _) = text_size(count_scale, font, &count_text);
    let count_x = overlay_x + (overlay_w.saturating_sub(count_width as u32)) / 2;
    let count_y = overlay_y + inner_margin as u32;
    draw_text_mut(&mut img, Rgba([0, 0, 0, 230]), count_x as i32, count_y as i32, count_scale, font, &count_text);

    // let specs_start_x = overlay_x + inner_margin as u32;
    // let specs_start_y = count_y + count_font_size as u32 + inner_margin as u32;
    // let line_spacing = specs_font_size as u32 + 24;
    // for (i, line) in specs_lines.iter().enumerate() {
    //     let y = specs_start_y + i as u32 * line_spacing;
    //     draw_text_mut(&mut img, Rgba([0, 0, 0, 230]), specs_start_x as i32, y as i32, specs_scale, font, line);
    // }

    // スペックについて、はみ出す場合は ... で省略
    let specs_start_x = overlay_x + inner_margin as u32;
    // カウントテキストの下の余ったスペースの中で上下中央に配置
    let specs_start_y = count_y + count_font_size as u32 + inner_margin as u32 + (overlay_h - count_y - count_font_size as u32 - inner_margin as u32 - specs_lines.len() as u32 * specs_font_size as u32) / 2;
    let line_spacing = specs_font_size as u32 + 24;
    let max_line_count = (overlay_h as f32 / line_spacing as f32).floor() as usize;
    for (i, line) in specs_lines.iter().enumerate().take(max_line_count) {
        let y = specs_start_y + i as u32 * line_spacing;
        draw_text_mut(&mut img, Rgba([0, 0, 0, 230]), specs_start_x as i32, y as i32, specs_scale, font, line);
    }
    if specs_lines.len() > max_line_count {
        let y = specs_start_y + max_line_count as u32 * line_spacing;
        let ellipsis = "...";
        draw_text_mut(&mut img, Rgba([0, 0, 0, 230]), specs_start_x as i32, y as i32, specs_scale, font, ellipsis);
    }

    img
}
