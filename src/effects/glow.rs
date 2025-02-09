use image::{Rgba, RgbaImage};
use crate::effects::Effect;

/// グローエフェクト（周囲に輝きを付与）
pub struct GlowEffect {
    pub range: i32,
    pub intensity: f32,
}

impl Default for GlowEffect {
    fn default() -> Self {
        Self { range: 3, intensity: 0.3 }
    }
}

impl Effect for GlowEffect {
    fn apply(
        &self,
        canvas: &mut RgbaImage,
        base_x: u32,
        base_y: u32,
        text_stamp: &RgbaImage,
        _cell_index: u32,
        _stamp_w: u32,
        _stamp_h: u32,
    ) {
        let glow_range = self.range;
        for dx in -glow_range..=glow_range {
            for dy in -glow_range..=glow_range {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                let alpha_factor = ((glow_range as f32 - dist) / glow_range as f32).max(0.0) * self.intensity;
                for (px, py, &p) in text_stamp.enumerate_pixels() {
                    let dest_x = base_x as i32 + dx + px as i32;
                    let dest_y = base_y as i32 + dy + py as i32;
                    if dest_x >= 0 && dest_y >= 0 && dest_x < canvas.width() as i32 && dest_y < canvas.height() as i32 {
                        let Rgba([r, g, b, a]) = p;
                        let white = 255u8;
                        let new_r = f32::from(r)
                            .mul_add(1.0 - alpha_factor, f32::from(white) * alpha_factor)
                            .min(255.0) as u8;
                        let new_g = f32::from(g)
                            .mul_add(1.0 - alpha_factor, f32::from(white) * alpha_factor)
                            .min(255.0) as u8;
                        let new_b = f32::from(b)
                            .mul_add(1.0 - alpha_factor, f32::from(white) * alpha_factor)
                            .min(255.0) as u8;
                        canvas.put_pixel(dest_x as u32, dest_y as u32, Rgba([new_r, new_g, new_b, a]));
                    }
                }
            }
        }
    }
}
