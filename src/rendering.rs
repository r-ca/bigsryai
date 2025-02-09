use image::{imageops, Rgba, RgbaImage};
use crate::effects::{Effect, EffectContext};

/// EffectContext とエフェクトの配列を受け取り、順次エフェクトを適用する
pub fn draw_cell_with_effects(context: &mut EffectContext, effects: &[Box<dyn Effect>]) {
    for effect in effects {
        effect.apply(context);
    }
}

/// 1 セル分の画像をレンダリングして返す
pub fn render_cell(
    cell_index: u32,
    text_stamp: &RgbaImage,
    stamp_w: u32,
    stamp_h: u32,
    effects: &[Box<dyn Effect>],
) -> RgbaImage {
    let mut cell_canvas = RgbaImage::from_pixel(stamp_w, stamp_h, Rgba([255, 255, 255, 255]));
    let mut context = EffectContext {
        canvas: &mut cell_canvas,
        base_x: 0,
        base_y: 0,
        text_stamp,
        cell_index,
        stamp_w,
        stamp_h,
    };
    draw_cell_with_effects(&mut context, effects);
    cell_canvas
}

/// 並列処理でセル画像を合成し、横一列の画像を生成する
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
