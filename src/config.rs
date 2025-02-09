pub struct Config {
    pub threshold_secs: f64,
    pub final_width: u32,
    pub final_height: u32,
    pub output_file: String,
    pub stamp_margin: u32,
    pub stamp_text: String,
    pub font_size: f32,
}

impl Config {
    pub fn new() -> Self {
        // 環境変数などから読み込むのではなく、固定値を集めています。
        Self {
            threshold_secs: 2.0,
            final_width: 1920,
            final_height: 1080,
            output_file: "output.png".to_string(),
            stamp_margin: 2,
            stamp_text: "nexryai".to_string(),
            font_size: 96.0,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
