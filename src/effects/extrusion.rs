use image::Rgba;
use crate::effects::{Effect, EffectContext};

/// 押し出し（エクストルージョン）エフェクト
pub struct ExtrusionEffect;

impl Effect for ExtrusionEffect {
    fn apply(&self, context: &mut EffectContext) {
        let extrude_steps = 8;
        let base_dx = 5;
        let base_dy = 2;
        for extrude in 0..extrude_steps {
            let off_x = context.base_x + extrude * base_dx;
            let off_y = context.base_y + extrude * base_dy;
            let dark_factor = 1.0 - (extrude as f32) / ((extrude_steps + 1) as f32);
            for (px, py, &p) in context.text_stamp.enumerate_pixels() {
                let distortion_x = 5.0 * (((px as f32) + context.cell_index as f32) * 0.17).sin();
                let distortion_y = 5.0 * (((py as f32) + context.cell_index as f32) * 0.17).cos();
                let dest_x = off_x + px + distortion_x.round() as u32;
                let dest_y = off_y + py + distortion_y.round() as u32;
                if dest_x < context.canvas.width() && dest_y < context.canvas.height() {
                    let Rgba([r, g, b, a]) = p;
                    let new_r = (f32::from(r) * dark_factor).min(255.0) as u8;
                    let new_g = (f32::from(g) * dark_factor).min(255.0) as u8;
                    let new_b = (f32::from(b) * dark_factor).min(255.0) as u8;
                    context.canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }
}
