use components::light::Light;

#[derive(Default, Clone)]
pub struct Illumination {
    pub sources: Vec<(Light, u32)>,
}
