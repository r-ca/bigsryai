use image::RgbaImage;

/// エフェクト共通トレイト
pub trait Effect: Send + Sync {
    /// キャンバス上の (base_x, base_y) から、text_stamp をもとにエフェクトを適用する
    fn apply(
        &self,
        canvas: &mut RgbaImage,
        base_x: u32,
        base_y: u32,
        text_stamp: &RgbaImage,
        cell_index: u32,
        stamp_w: u32,
        stamp_h: u32,
    );
}

pub mod glow;
pub mod extrusion;
pub mod rotation;
pub mod sparkle;
pub mod surreal;
pub mod colorful;

pub use glow::GlowEffect;
pub use extrusion::ExtrusionEffect;
pub use rotation::RotationEffect;
pub use sparkle::SparkleEffect;
pub use surreal::SurrealEffect;
pub use colorful::ColorfulEffect;
