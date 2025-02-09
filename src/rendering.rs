use image::{imageops, Rgba, RgbaImage};
use crate::effects::{Effect, EffectContext};
use rayon::prelude::*;
use std::time::Instant;

/// 各エフェクトを順次適用する
pub fn draw_cell_with_effects(context: &mut EffectContext, effects: &[Box<dyn Effect>]) {
    for effect in effects {
        effect.apply(context);
    }
}

/// 1セル分の画像をレンダリングして返す
pub fn render_cell(
    cell_index: u32,
    text_stamp: &RgbaImage,
    stamp_w: u32,
    stamp_h: u32,
    effects: &[Box<dyn Effect>],
) -> RgbaImage {
    let mut cell_canvas =
        RgbaImage::from_pixel(stamp_w, stamp_h, Rgba([255, 255, 255, 255]));
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

/// 並列処理でセル画像を生成し、横一列の画像を作成する
pub fn benchmark_render(
    letter_count: u32,
    stamp_w: u32,
    stamp_h: u32,
    text_stamp: &RgbaImage,
    effects: &[Box<dyn Effect>],
) -> (std::time::Duration, RgbaImage) {
    let start = Instant::now();
    let cell_images: Vec<RgbaImage> = (0..letter_count)
        .into_par_iter()
        .map(|i| render_cell(i, text_stamp, stamp_w, stamp_h, effects))
        .collect();
    let final_w = letter_count * stamp_w;
    let final_h = stamp_h;
    let mut canvas =
        RgbaImage::from_pixel(final_w, final_h, Rgba([255, 255, 255, 255]));
    for (i, cell) in cell_images.into_iter().enumerate() {
        let dest_x = i as u32 * stamp_w;
        imageops::overlay(&mut canvas, &cell, dest_x as i64, 0);
    }
    (start.elapsed(), canvas)
}

/// 横一列キャンバスを、左上からセルを順次配置して並べ直すことで、
/// 16:9に近い（かつ全体を埋める）グリッド状に配置し、
/// 最後に多少の変形（リサイズ）で1920×1080の画像を作成する
///
/// この関数では、まず横一列に並んだセル画像（各サイズ: stamp_w x stamp_h）を
/// 最適な行数・列数（グリッド）の候補を探索します。グリッドのアスペクト比が
/// ターゲット（1920/1080＝16/9）に近いものを選び、
/// 左上から順次セルを配置した後、最終的にLanczos3フィルターでリサイズします。
pub fn rearrange_by_flat_pixels(
    horizontal_canvas: &RgbaImage,
    final_w: u32,
    final_h: u32,
    stamp_w: u32,
    stamp_h: u32,
) -> RgbaImage {
    // 横一列キャンバスの高さは stamp_h である前提
    // セル数は、横幅 / stamp_w で求められる（整数で割り切れるものとする）
    let total_cells = horizontal_canvas.width() / stamp_w;
    let target_aspect = final_w as f32 / final_h as f32;
    let mut best_diff = f32::MAX;
    let mut best_cols = 1;
    // 1セルから全セルまで、列数を変えてグリッドのアスペクト比を評価
    for cols in 1..=total_cells {
        let rows = ((total_cells as f32) / (cols as f32)).ceil() as u32;
        let grid_aspect = (cols as f32 * stamp_w as f32) / (rows as f32 * stamp_h as f32);
        let diff = (grid_aspect - target_aspect).abs();
        if diff < best_diff {
            best_diff = diff;
            best_cols = cols;
        }
    }
    let grid_cols = best_cols;
    let grid_rows = ((total_cells as f32) / (grid_cols as f32)).ceil() as u32;
    let arranged_width = grid_cols * stamp_w;
    let arranged_height = grid_rows * stamp_h;
    let mut arranged_image =
        RgbaImage::from_pixel(arranged_width, arranged_height, Rgba([255, 255, 255, 255]));
    
    // 横一列キャンバスは、セルが左から右に並んでいるので、セル単位で切り出し、
    // 左上から行単位（row-major order）で配置する
    for i in 0..total_cells {
        let src_x = i * stamp_w;
        let cell = imageops::crop_imm(horizontal_canvas, src_x, 0, stamp_w, stamp_h).to_image();
        let dest_x = (i % grid_cols) * stamp_w;
        let dest_y = (i / grid_cols) * stamp_h;
        imageops::overlay(&mut arranged_image, &cell, dest_x as i64, dest_y as i64);
    }
    
    // 配置したグリッド画像を、最終的に1920×1080のサイズにリサイズ（多少の変形を許容）
    imageops::resize(
        &arranged_image,
        final_w,
        final_h,
        imageops::FilterType::Lanczos3,
    )
}
