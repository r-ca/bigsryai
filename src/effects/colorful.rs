use image::{Rgba, RgbaImage};
use crate::effects::Effect;
use crate::text::hsv_to_rgb; // text.rs 内の hsv_to_rgb を利用

/// カラフルな乱れエフェクト
pub struct ColorfulEffect;

impl Effect for ColorfulEffect {
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
        for (px, py, &p) in text_stamp.enumerate_pixels() {
            if (px + py + cell_index) % 7 == 0 {
                let offset_x = (5.0 * ((px as f32 + cell_index as f32) * 0.27).sin()).round() as i32;
                let offset_y = (5.0 * ((py as f32 + cell_index as f32) * 0.27).cos()).round() as i32;
                let dest_x = base_x as i32 + px as i32 + offset_x;
                let dest_y = base_y as i32 + py as i32 + offset_y;
                if dest_x >= 0 && dest_y >= 0 &&
                   dest_x < canvas.width() as i32 &&
                   dest_y < canvas.height() as i32 {
                    let hue = (cell_index as f32 * 0.05
                        + (px as f32 / stamp_w as f32)
                        + (py as f32 / stamp_h as f32)) % 1.0;
                    let (r2, g2, b2) = hsv_to_rgb(hue, 0.9, 1.0);
                    let Rgba([r, g, b, a]) = p;
                    let new_r = ((u16::from(r) + u16::from(r2)) / 2) as u8;
                    let new_g = ((u16::from(g) + u16::from(g2)) / 2) as u8;
                    let new_b = ((u16::from(b) + u16::from(b2)) / 2) as u8;
                    canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
                }
            }
        }
    }
}
