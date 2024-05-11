pub struct Image {
    image: image::RgbaImage,
}
impl Image {
    pub fn new_from_memory(data: &[u8]) -> Self {
        let image = image::load_from_memory(data).unwrap_or_else(|error| image::DynamicImage::new_rgba8(1, 1));
        let image = image.to_rgba8();
        Self {
            image
        }
    }
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn bytes(&self) -> &[u8] {
        self.image.as_raw()
    }

}
