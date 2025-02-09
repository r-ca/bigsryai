use image::{Rgba, RgbaImage};
use crate::effects::Effect;

/// 押し出し（エクストルージョン）エフェクト
pub struct ExtrusionEffect;

impl Effect for ExtrusionEffect {
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
        let extrude_steps = 8;
        let base_dx = 5;
        let base_dy = 2;
        for extrude in 0..extrude_steps {
            let off_x = base_x + extrude * base_dx;
            let off_y = base_y + extrude * base_dy;
            let dark_factor = 1.0 - (extrude as f32) / ((extrude_steps + 1) as f32);
            for (px, py, &p) in text_stamp.enumerate_pixels() {
                let distortion_x = 5.0 * (((px as f32) + cell_index as f32) * 0.17).sin();
                let distortion_y = 5.0 * (((py as f32) + cell_index as f32) * 0.17).cos();
                let dest_x = off_x + px + distortion_x.round() as u32;
                let dest_y = off_y + py + distortion_y.round() as u32;
                if dest_x < canvas.width() && dest_y < canvas.height() {
                    let Rgba([r, g, b, a]) = p;
                    let new_r = (f32::from(r) * dark_factor).min(255.0) as u8;
                    let new_g = (f32::from(g) * dark_factor).min(255.0) as u8;
                    let new_b = (f32::from(b) * dark_factor).min(255.0) as u8;
                    canvas.put_pixel(dest_x, dest_y, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }
}
