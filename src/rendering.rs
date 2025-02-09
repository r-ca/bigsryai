use image::{imageops, Rgba, RgbaImage};
use crate::effects::{Effect, EffectContext};

/// EffectContext とエフェクトの配列を受け取り、順次エフェクトを適用する
pub fn draw_cell_with_effects(context: &mut EffectContext, effects: &[Box<dyn Effect>]) {
    for effect in effects {
        effect.apply(context);
    }
}

/// 1 セル分の画像をレンダリングして返す
pub fn render_cell(
    cell_index: u32,
    text_stamp: &RgbaImage,
    stamp_w: u32,
    stamp_h: u32,
    effects: &[Box<dyn Effect>],
) -> RgbaImage {
    let mut cell_canvas = RgbaImage::from_pixel(stamp_w, stamp_h, Rgba([255, 255, 255, 255]));
    let mut context = EffectContext {
        canvas: &mut cell_canvas,
        base_x: 0,
        base_y: 0,
        text_stamp,
        cell_index,
        stamp_w,
        stamp_h,
    };
    draw_cell_with_effects(&mut context, effects);
    cell_canvas
}

/// 並列処理でセル画像を合成し、横一列の画像を生成する
pub fn benchmark_render(
    letter_count: u32,
    stamp_w: u32,
    stamp_h: u32,
    text_stamp: &RgbaImage,
    effects: &[Box<dyn Effect>],
) -> (std::time::Duration, RgbaImage) {
    use rayon::prelude::*;
    let start = std::time::Instant::now();
    let cell_images: Vec<RgbaImage> = (0..letter_count)
        .into_par_iter()
        .map(|i| render_cell(i, text_stamp, stamp_w, stamp_h, effects))
        .collect();
    let final_w = letter_count * stamp_w;
    let final_h = stamp_h;
    let mut canvas = RgbaImage::from_pixel(final_w, final_h, Rgba([255, 255, 255, 255]));
    for (i, cell) in cell_images.into_iter().enumerate() {
        let dest_x = i as u32 * stamp_w;
        imageops::overlay(&mut canvas, &cell, dest_x as i64, 0);
    }
    (start.elapsed(), canvas)
}

/// 横一列キャンバス（幅 = letter_count * stamp_w, 高さ = stamp_h）から、
/// スタンプの縦幅（stamp_h）はそのまま保持し、
/// 横方向は全ピクセルを連続したストリームとみなして、
/// 指定の横幅で改行（＝新行の長さ L ピクセルで分割）して再配置する関数です。
///
/// ※新グリッド画像の横幅 grid_width = L
///  新グリッド画像の縦幅 grid_height = (ceil(horizontal.width() / L)) * stamp_h
///
/// ここで L は以下の式により求めています：
///  L ≒ sqrt((16/9) × (horizontal.width() * stamp_h))
///
/// 最後に、grid_image を Lanczos3 フィルターで指定サイズ（例：1920×1080）にリサイズして返します。
pub fn rearrange_by_flat_pixels(
    horizontal: &RgbaImage,
    final_width: u32,
    final_height: u32,
    stamp_h: u32,
) -> RgbaImage {
    // 横一列キャンバスの横幅（＝ letter_count * stamp_w）
    let horizontal_width = horizontal.width();
    // 総ピクセル数（横一列キャンバス全体）
    let total_pixels = (horizontal_width * horizontal.height()) as usize; // horizontal.height() = stamp_h

    // 横方向はスタンプ境界を一切無視し、全ピクセル数と stamp_h を使って新たな横幅 L を求める
    let ideal_l = ((16.0 / 9.0) * (horizontal_width as f64) * (stamp_h as f64)).sqrt();
    let L = ideal_l.round() as u32;
    let new_width = L.max(1);

    // 新画像の行数 = ceil(horizontal_width / new_width)
    let num_lines = ((horizontal_width as f64) / (new_width as f64)).ceil() as u32;
    let grid_height = num_lines * stamp_h;
    
    // 新グリッド画像（背景は白）
    let mut grid_image = RgbaImage::from_pixel(new_width, grid_height, Rgba([255, 255, 255, 255]));
    
    // 横一列キャンバスをフラットなピクセル列とみなし、row-major 順に Vec に展開
    let pixels: Vec<Rgba<u8>> = horizontal.pixels().cloned().collect();
    
    // 新グリッド画像は「テキスト」を再構成するように、1行あたり new_width ピクセル、全体で num_lines 行
    // ただし各「行」は stamp_h ピクセルの高さ（つまりスタンプの縦幅はそのまま）
    for i in 0..(new_width as usize * grid_height as usize) {
        let src_idx = i % total_pixels;
        let x = (i % new_width as usize) as u32;
        let y = (i / new_width as usize) as u32;
        grid_image.put_pixel(x, y, pixels[src_idx]);
    }
    
    // 最終的に、得られたグリッド画像を指定の最終解像度にリサイズして返す
    imageops::resize(&grid_image, final_width, final_height, imageops::Lanczos3)
}
