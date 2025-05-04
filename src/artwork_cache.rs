use crate::artwork::Artwork;

pub struct ArtworkCache(Option<Artwork>);
impl ArtworkCache {
    pub fn get(&mut self) -> Result<Artwork, anyhow::Error> {
        if self.0.is_some() {
            return Ok(self.0.clone().unwrap());
        }
        let new_artwork = Artwork::try_init()?;
        self.0.replace(new_artwork);
        return Ok(self.0.clone().unwrap());
    }

    pub fn clear(&mut self) -> &mut ArtworkCache {
        self.0 = None;
        return self;
    }

    pub fn init() -> Self {
        return Self(None);
    }
}
