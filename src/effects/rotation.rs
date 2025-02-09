use image::{Rgba, RgbaImage};
use crate::effects::Effect;

/// 回転・変形エフェクト
pub struct RotationEffect;

impl Effect for RotationEffect {
    fn apply(
        &self,
        canvas: &mut RgbaImage,
        base_x: u32,
        base_y: u32,
        text_stamp: &RgbaImage,
        cell_index: u32,
        stamp_w: u32,
        stamp_h: u32,
    ) {
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
            let new_y = center_y + rdy * 0.7;
            let dest_x = base_x + new_x.round() as u32;
            let dest_y = base_y + new_y.round() as u32;
            if dest_x < canvas.width() && dest_y < canvas.height() {
                let Rgba([r, g, b, a]) = p;
                let new_r = r.saturating_add(30);
                canvas.put_pixel(dest_x, dest_y, Rgba([new_r, g, b, a]));
            }
        }
    }
}
