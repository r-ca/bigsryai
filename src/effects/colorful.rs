use image::Rgba;
use crate::effects::{Effect, EffectContext};
use crate::text::hsv_to_rgb; // text.rs 内の hsv_to_rgb を利用

/// カラフルな乱れエフェクト
pub struct ColorfulEffect;

impl Effect for ColorfulEffect {
    fn apply(&self, context: &mut EffectContext) {
        for (px, py, &p) in context.text_stamp.enumerate_pixels() {
            if (px + py + context.cell_index) % 7 == 0 {
                let offset_x = (5.0 * ((px as f32 + context.cell_index as f32) * 0.27).sin()).round() as i32;
                let offset_y = (5.0 * ((py as f32 + context.cell_index as f32) * 0.27).cos()).round() as i32;
                let dest_x = context.base_x as i32 + px as i32 + offset_x;
                let dest_y = context.base_y as i32 + py as i32 + offset_y;
                if dest_x >= 0 && dest_y >= 0 &&
                   dest_x < context.canvas.width() as i32 &&
                   dest_y < context.canvas.height() as i32 {
                    let hue = (context.cell_index as f32 * 0.05
                        + (px as f32 / context.stamp_w as f32)
                        + (py as f32 / context.stamp_h as f32)) % 1.0;
                    let (r2, g2, b2) = hsv_to_rgb(hue, 0.9, 1.0);
                    let Rgba([r, g, b, a]) = p;
                    let new_r = ((u16::from(r) + u16::from(r2)) / 2) as u8;
                    let new_g = ((u16::from(g) + u16::from(g2)) / 2) as u8;
                    let new_b = ((u16::from(b) + u16::from(b2)) / 2) as u8;
                    context.canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }
}
