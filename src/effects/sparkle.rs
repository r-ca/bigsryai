use image::{Rgba, RgbaImage};
use crate::effects::Effect;

/// Sparkle（輝き）エフェクト
pub struct SparkleEffect;

impl Effect for SparkleEffect {
    fn apply(
        &self,
        canvas: &mut RgbaImage,
        base_x: u32,
        base_y: u32,
        text_stamp: &RgbaImage,
        cell_index: u32,
        _stamp_w: u32,
        _stamp_h: u32,
    ) {
        for (px, py, &p) in text_stamp.enumerate_pixels() {
            let Rgba([r, g, b, a]) = p;
            let lum = (u32::from(r) + u32::from(g) + u32::from(b)) / 3;
            if lum > 200 && ((px + py + cell_index) % 97 == 0) {
                let dest_x = base_x + px;
                let dest_y = base_y + py;
                if dest_x < canvas.width() && dest_y < canvas.height() {
                    canvas.put_pixel(dest_x, dest_y, Rgba([255, 255, 255, a]));
                }
            }
        }
    }
}
