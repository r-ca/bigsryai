use image::{Rgba, RgbaImage};
use image::imageops::overlay;
use imageproc::drawing::draw_text_mut;
use ab_glyph::{FontArc, PxScale};
use sysinfo::System;

pub fn annotate_image(mut img: RgbaImage, count: u32, font: &FontArc, sys: &System) -> RgbaImage {
    // オーバーレイ領域の設定
    let (overlay_x, overlay_y, overlay_w, overlay_h) = {
        let margin_x = img.width() / 10;
        let margin_y = img.height() / 10;
        (
            margin_x,
            margin_y,
            img.width() - 2 * margin_x,
            img.height() - 2 * margin_y,
        )
    };

    // 半透明オーバーレイ背景の作成
    let overlay_bg = RgbaImage::from_pixel(overlay_w, overlay_h, Rgba([255, 255, 255, 128]));
    overlay(&mut img, &overlay_bg, overlay_x.into(), overlay_y.into());

    // システム情報の取得
    let specs_lines = create_system_info_lines(sys);
    let count_text = count.to_string();

    // テキスト描画処理
    draw_overlay_text(
        &mut img,
        overlay_x,
        overlay_y,
        overlay_w,
        overlay_h,
        &count_text,
        &specs_lines,
        font,
    );

    img
}

/// システム情報文字列の生成
fn create_system_info_lines(sys: &System) -> Vec<String> {
    let hostname = sys.host_name().unwrap_or_else(|| "Unknown".into());
    let os_info = sys.long_os_version().unwrap_or_else(|| "Unknown OS".into());
    
    let cpu_info = sys.cpus().first().map(|cpu| {
        match (cpu.brand(), cpu.name()) {
            (brand, name) if brand == name => brand.to_string(),
            (brand, name) => format!("{brand} ({name})"),
        }
    }).unwrap_or_else(|| "Unknown CPU".into());

    let memory_info = {
        let total = sys.total_memory() as f64 / 1_048_576.0;  // KB to GB
        let used = sys.used_memory() as f64 / 1_048_576.0;
        format!("Memory: {used:.2} GB / {total:.2} GB")
    };

    vec![
        format!("Hostname: {hostname}"),
        format!("CPU: {} ({} cores)", cpu_info, sys.cpus().len()),
        format!("OS: {os_info}"),
        memory_info,
    ]
}

/// オーバーレイテキスト描画関数
fn draw_overlay_text(
    img: &mut RgbaImage,
    overlay_x: u32,
    overlay_y: u32,
    overlay_w: u32,
    overlay_h: u32,
    count_text: &str,
    specs_lines: &[String],
    font: &FontArc,
) {
    // フォントサイズ設定
    let count_scale = PxScale {
        x: overlay_h as f32 * 0.2,
        y: overlay_h as f32 * 0.2,
    };
    let specs_scale = PxScale {
        x: overlay_h as f32 * 0.05,
        y: overlay_h as f32 * 0.05,
    };

    // カウントテキスト描画
    let (count_x, count_y) = calculate_centered_position(
        overlay_x,
        overlay_y,
        overlay_w,
        count_text,
        count_scale,
        font,
    );
    draw_text_mut(
        img,
        Rgba([0, 0, 0, 230]),
        count_x as i32,
        count_y as i32,
        count_scale,
        font,
        count_text,
    );

    // スペックテキスト描画
    let inner_margin = (overlay_w as f32 * 0.05) as u32;
    let line_spacing = specs_scale.y as u32 + 24;
    let max_lines = (overlay_h as f32 / line_spacing as f32).floor() as usize;

    let specs_start_y = calculate_specs_start_y(
        count_y,
        count_scale.y as u32,
        overlay_h,
        inner_margin,
        specs_lines.len(),
        specs_scale.y as u32,
    );

    specs_lines.iter().take(max_lines).enumerate().for_each(|(i, line)| {
        draw_text_mut(
            img,
            Rgba([0, 0, 0, 230]),
            (overlay_x + inner_margin) as i32,
            (specs_start_y + i as u32 * line_spacing) as i32,
            specs_scale,
            font,
            line,
        );
    });

    // 省略記号の描画
    if specs_lines.len() > max_lines {
        draw_text_mut(
            img,
            Rgba([0, 0, 0, 230]),
            (overlay_x + inner_margin) as i32,
            (specs_start_y + max_lines as u32 * line_spacing) as i32,
            specs_scale,
            font,
            "...",
        );
    }
}

/// 中央揃え位置計算
fn calculate_centered_position(
    x: u32,
    y: u32,
    width: u32,
    text: &str,
    scale: PxScale,
    font: &FontArc,
) -> (u32, u32) {
    let (text_width, _) = imageproc::drawing::text_size(scale, font, text);
    (
        x + (width.saturating_sub(text_width as u32)) / 2,
        y + (width as f32 * 0.05) as u32,
    )
}

/// スペックテキスト開始Y位置計算
fn calculate_specs_start_y(
    count_y: u32,
    count_height: u32,
    overlay_h: u32,
    margin: u32,
    line_count: usize,
    line_height: u32,
) -> u32 {
    let remaining_space = overlay_h
        .saturating_sub(count_y)
        .saturating_sub(count_height)
        .saturating_sub(margin);

    let content_height = line_count as u32 * line_height;
    let vertical_padding = remaining_space.saturating_sub(content_height) / 2;

    count_y + count_height + margin + vertical_padding
}
