use image::{imageops, GenericImageView, Rgba, RgbaImage};
use crate::effects::{Effect, EffectContext};

pub fn draw_cell_with_effects(context: &mut EffectContext, effects: &[Box<dyn Effect>]) {
    for effect in effects {
        effect.apply(context);
    }
}

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

pub fn rearrange_to_aspect(
    horizontal: &RgbaImage,
    stamp_w: u32,
    stamp_h: u32,
    letter_count: u32,
    final_width: u32,
    final_height: u32,
) -> RgbaImage {
    let target_aspect = 16.0 / 9.0;
    let mut best_columns = 1;
    let mut best_diff = f64::MAX;
    for columns in 1..=letter_count {
        let rows = ((letter_count as f64) / (columns as f64)).ceil() as u32;
        let grid_width = columns * stamp_w;
        let grid_height = rows * stamp_h;
        let aspect = grid_width as f64 / grid_height as f64;
        let diff = (aspect - target_aspect).abs();
        if diff < best_diff {
            best_diff = diff;
            best_columns = columns;
        }
    }
    let rows = ((letter_count as f64) / (best_columns as f64)).ceil() as u32;
    let grid_width = best_columns * stamp_w;
    let grid_height = rows * stamp_h;
    let mut grid_image = RgbaImage::from_pixel(grid_width, grid_height, Rgba([255, 255, 255, 255]));
    for i in 0..letter_count {
        let src_x = i * stamp_w;
        let stamp = horizontal.view(src_x, 0, stamp_w, stamp_h).to_image();
        let col = i % best_columns;
        let row = i / best_columns;
        let dest_x = col * stamp_w;
        let dest_y = row * stamp_h;
        imageops::overlay(&mut grid_image, &stamp, dest_x as i64, dest_y as i64);
    }
    imageops::resize(&grid_image, final_width, final_height, imageops::Lanczos3)
}
