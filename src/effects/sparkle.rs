use image::{Rgba, RgbaImage};
use crate::effects::{Effect, EffectContext};

/// Sparkle（輝き）エフェクト
pub struct SparkleEffect;

impl Effect for SparkleEffect {
    fn apply(&self, context: &mut EffectContext) {
        for (px, py, &p) in context.text_stamp.enumerate_pixels() {
            let Rgba([r, g, b, a]) = p;
            let lum = (u32::from(r) + u32::from(g) + u32::from(b)) / 3;
            if lum > 200 && ((px + py + context.cell_index) % 97 == 0) {
                let dest_x = context.base_x + px;
                let dest_y = context.base_y + py;
                if dest_x < context.canvas.width() && dest_y < context.canvas.height() {
                    context.canvas.put_pixel(dest_x, dest_y, Rgba([255, 255, 255, a]));
                }
            }
        }
    }
}
