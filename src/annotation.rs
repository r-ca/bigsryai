use image::{Rgba, RgbaImage};
use image::imageops;
use rusttype::{Font, Scale, point};
use sysinfo::System;

pub fn annotate_image(mut img: RgbaImage, count: u32, font: &Font, sys: &System) -> RgbaImage {
    // --- オーバーレイ領域の作成 ---
    let overlay_margin_x = img.width() / 10;
    let overlay_margin_y = img.height() / 10;
    let overlay_x = overlay_margin_x;
    let overlay_y = overlay_margin_y;
    let overlay_w = img.width() - 2 * overlay_margin_x;
    let overlay_h = img.height() - 2 * overlay_margin_y;

    // 薄い白（アルファ 200/255）のオーバーレイ画像を作成し、ブラー（σ=5.0）をかける
    let overlay = RgbaImage::from_pixel(overlay_w, overlay_h, Rgba([255, 255, 255, 200]));
    let blurred_overlay = imageops::blur(&overlay, 5.0);
    imageops::overlay(&mut img, &blurred_overlay, overlay_x as i64, overlay_y as i64);

    // --- sysinfo からシステム情報を取得 ---
    // すでに外部で refresh_all() 済みの System インスタンスを使用する
    let hostname = sys.host_name().unwrap_or_else(|| "Unknown".to_string());
    let os_info = sys.long_os_version().unwrap_or_else(|| "Unknown OS".to_string());
    let cpu_name = sys.global_cpu_info().brand().to_string();
    let core_count = sys.cpus().len();
    let total_memory_kb = sys.total_memory();
    let used_memory_kb = sys.used_memory();

    // メモリは KB 単位から MB, GB に変換（MB は整数、GB は小数点2桁）
    let total_memory_mb = total_memory_kb as f64 / 1024.0;
    let used_memory_mb = used_memory_kb as f64 / 1024.0;
    let total_memory_gb = total_memory_mb / 1024.0;
    let used_memory_gb = used_memory_mb / 1024.0;

    // --- 表示するテキストの作成 ---
    // カウント（数字のみ）は大きく中央に表示
    let count_text = count.to_string();
    // スペック情報は左揃えで表示
    let specs_lines = vec![
        format!("Hostname: {}", hostname),
        format!("CPU: {} ({} cores)", cpu_name, core_count),
        format!("OS: {}", os_info),
        format!(
            "Memory: {:.2} GB ({:.0} MB) / {:.2} GB ({:.0} MB)",
            used_memory_gb, used_memory_mb, total_memory_gb, total_memory_mb
        ),
    ];

    // --- テキスト描画（imageproc を使用） ---
    use imageproc::drawing::draw_text_mut;
    let text_color = Rgba([0, 0, 0, 255]); // 黒

    // カウント用のフォントは大きめ（例：64pt）
    let count_scale = Scale::uniform(64.0);
    // スペック用のフォントは小さめ（例：24pt）
    let specs_scale = Scale::uniform(24.0);

    // カウントテキストはオーバーレイ内で水平中央に配置
    let v_metrics_count = font.v_metrics(count_scale);
    let glyphs_count: Vec<_> = font.layout(&count_text, count_scale, point(0.0, v_metrics_count.ascent)).collect();
    let count_width = glyphs_count
        .iter()
        .rev()
        .find_map(|g| g.pixel_bounding_box().map(|bb| bb.max.x as u32))
        .unwrap_or(0);
    let count_x = overlay_x + (overlay_w.saturating_sub(count_width)) / 2;
    let count_y = overlay_y + 20; // 上からの余白

    draw_text_mut(&mut img, text_color, count_x as i32, count_y as i32, count_scale, font, &count_text);

    // スペック情報はカウントの下部、オーバーレイ内の左側に配置
    let specs_start_x = overlay_x + 20;  // 左から20px余白
    let specs_start_y = count_y + 80;      // カウント下から約80px
    let line_spacing = 30;               // 行間30px

    for (i, line) in specs_lines.iter().enumerate() {
        let y = specs_start_y + i as u32 * line_spacing;
        draw_text_mut(&mut img, text_color, specs_start_x as i32, y as i32, specs_scale, font, line);
    }

    img
}
