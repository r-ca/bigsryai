use image::{Rgba, RgbaImage};
use rusttype::{Font, Scale, point};

/// HSV から RGB への変換
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0.0, 0.0, 0.0),
    };
    ((r * 255.0).round() as u8,
     (g * 255.0).round() as u8,
     (b * 255.0).round() as u8)
}

/// フォントからテキストスタンプ画像を生成する  
/// Config から渡された font_size を利用してテキストの大きさを設定
pub fn generate_stamp(font: &Font, text: &str, margin: u32, font_size: f32) -> RgbaImage {
    let scale = Scale::uniform(font_size);
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();
    let min_x = glyphs.iter()
        .filter_map(|g| g.pixel_bounding_box().map(|bb| bb.min.x))
        .min()
        .unwrap_or(0);
    let min_y = glyphs.iter()
        .filter_map(|g| g.pixel_bounding_box().map(|bb| bb.min.y))
        .min()
        .unwrap_or(0);
    let max_x = glyphs.iter()
        .filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.x))
        .max()
        .unwrap_or(0);
    let max_y = glyphs.iter()
        .filter_map(|g| g.pixel_bounding_box().map(|bb| bb.max.y))
        .max()
        .unwrap_or(0);
    let text_width = (max_x - min_x) as u32;
    let text_height = (max_y - min_y) as u32;
    let stamp_width = text_width + 2 * margin;
    let stamp_height = text_height + 2 * margin;
    let mut text_stamp = RgbaImage::from_pixel(stamp_width, stamp_height, Rgba([255, 255, 255, 255]));
    let offset = point(margin as f32 - min_x as f32, margin as f32 - min_y as f32 + v_metrics.ascent);
    for glyph in font.layout(text, scale, offset) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let x = bb.min.x + gx as i32;
                let y = bb.min.y + gy as i32;
                if x >= 0 && y >= 0 && (x as u32) < stamp_width && (y as u32) < stamp_height {
                    let hue = (x as f32) / (stamp_width as f32);
                    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                    let blended_r = f32::from(r).mul_add(v, 255.0 * (1.0 - v)).round() as u8;
                    let blended_g = f32::from(g).mul_add(v, 255.0 * (1.0 - v)).round() as u8;
                    let blended_b = f32::from(b).mul_add(v, 255.0 * (1.0 - v)).round() as u8;
                    text_stamp.put_pixel(x as u32, y as u32, Rgba([blended_r, blended_g, blended_b, 255]));
                }
            });
        }
    }
    text_stamp
}
