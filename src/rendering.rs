use image::{imageops, Rgba, RgbaImage};
use crate::effects::Effect;

/// 複数エフェクトを順に適用してセル内に描画する
pub fn draw_cell_with_effects(
    canvas: &mut RgbaImage,
    base_x: u32,
    base_y: u32,
    text_stamp: &RgbaImage,
    cell_index: u32,
    stamp_w: u32,
    stamp_h: u32,
    effects: &[Box<dyn Effect>],
) {
    for effect in effects.iter() {
        effect.apply(canvas, base_x, base_y, text_stamp, cell_index, stamp_w, stamp_h);
    }
}

/// 1セル分の画像をレンダリングして返す
pub fn render_cell(
    cell_index: u32,
    text_stamp: &RgbaImage,
    stamp_w: u32,
    stamp_h: u32,
    effects: &[Box<dyn Effect>],
) -> RgbaImage {
    let mut cell_canvas = RgbaImage::from_pixel(stamp_w, stamp_h, Rgba([255, 255, 255, 255]));
    draw_cell_with_effects(&mut cell_canvas, 0, 0, text_stamp, cell_index, stamp_w, stamp_h, effects);
    cell_canvas
}

/// 並列処理でセル画像を合成して横一列の画像を生成する
pub fn benchmark_render(
    letter_count: u32,
    stamp_w: u32,
    stamp_h: u32,
    text_stamp: &RgbaImage,
    effects: &[Box<dyn Effect>],
) -> (std::time::Duration, RgbaImage) {
    use rayon::prelude::*;
    let start = std::time::Instant::now();
    let cell_images: Vec<RgbaImage> = (0..letter_count)
        .into_par_iter()
        .map(|i| render_cell(i, text_stamp, stamp_w, stamp_h, effects))
        .collect();
    let final_w = letter_count * stamp_w;
    let final_h = stamp_h;
    let mut canvas = RgbaImage::from_pixel(final_w, final_h, Rgba([255, 255, 255, 255]));
    for (i, cell) in cell_images.into_iter().enumerate() {
        let dest_x = i as u32 * stamp_w;
        imageops::overlay(&mut canvas, &cell, dest_x as i64, 0);
    }
    (start.elapsed(), canvas)
}
