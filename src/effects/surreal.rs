use image::{Rgba, RgbaImage};
use crate::effects::Effect;

/// シュールな右シフトエフェクト
pub struct SurrealEffect;

impl Effect for SurrealEffect {
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
            if (px + py + cell_index) % 101 == 0 {
                let dest_x = base_x + px + 3;
                let dest_y = base_y + py;
                if dest_x < canvas.width() && dest_y < canvas.height() {
                    let Rgba([r, g, b, a]) = p;
                    let new_r = r.saturating_sub(10);
                    let new_g = g.saturating_sub(10);
                    let new_b = ((u16::from(b) + 10).min(255)) as u8;
                    canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }
}
