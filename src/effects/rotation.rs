use image::{Rgba, RgbaImage};
use crate::effects::{Effect, EffectContext};

/// 回転・変形エフェクト
pub struct RotationEffect;

impl Effect for RotationEffect {
    fn apply(&self, context: &mut EffectContext) {
        let center_x = context.stamp_w as f32 / 2.0;
        let center_y = context.stamp_h as f32 / 2.0;
        let angle = 0.3 * ((context.cell_index as f32 * 0.7).sin());
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        for (px, py, &p) in context.text_stamp.enumerate_pixels() {
            let dx = px as f32 - center_x;
            let dy = py as f32 - center_y;
            let rdx = dx * cos_a - dy * sin_a;
            let rdy = dx * sin_a + dy * cos_a;
            let new_x = center_x + rdx;
            let new_y = center_y + rdy * 0.7;
            let dest_x = context.base_x + new_x.round() as u32;
            let dest_y = context.base_y + new_y.round() as u32;
            if dest_x < context.canvas.width() && dest_y < context.canvas.height() {
                let Rgba([r, g, b, a]) = p;
                let new_r = r.saturating_add(30);
                context.canvas.put_pixel(dest_x, dest_y, Rgba([new_r, g, b, a]));
            }
        }
    }
}
