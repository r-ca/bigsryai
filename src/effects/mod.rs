use image::RgbaImage;

/// エフェクト適用時に共通で利用する情報をまとめたコンテキスト
pub struct EffectContext<'a> {
    pub canvas: &'a mut RgbaImage,
    pub base_x: u32,
    pub base_y: u32,
    pub text_stamp: &'a RgbaImage,
    pub cell_index: u32,
    pub stamp_w: u32,
    pub stamp_h: u32,
}

/// エフェクト共通トレイト
/// `Send + Sync` を付与してスレッド間でも安全に扱えるようにします
pub trait Effect: Send + Sync {
    /// EffectContext をもとにエフェクトを適用する
    fn apply(&self, context: &mut EffectContext);
}

pub mod glow;
pub mod extrusion;
pub mod rotation;
pub mod sparkle;
pub mod surreal;
pub mod colorful;

