use image::Rgba;
use crate::effects::{Effect, EffectContext};

/// シュールな右シフトエフェクト
pub struct SurrealEffect;

impl Effect for SurrealEffect {
    fn apply(&self, context: &mut EffectContext) {
        for (px, py, &p) in context.text_stamp.enumerate_pixels() {
            if (px + py + context.cell_index) % 101 == 0 {
                let dest_x = context.base_x + px + 3;
                let dest_y = context.base_y + py;
                if dest_x < context.canvas.width() && dest_y < context.canvas.height() {
                    let Rgba([r, g, b, a]) = p;
                    let new_r = r.saturating_sub(10);
                    let new_g = g.saturating_sub(10);
                    let new_b = ((u16::from(b) + 10).min(255)) as u8;
                    context.canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }
}
