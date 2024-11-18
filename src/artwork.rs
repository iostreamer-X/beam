use serde::Serialize;

#[cfg(test)]
use mockall::automock;
#[cfg_attr(test, automock)]
pub trait ArtworkFetcher {
    fn get_artwork_string(&self) -> Option<String>;
}

#[derive(Debug, Default, Serialize)]
pub struct Artwork(Option<String>);
impl Artwork {
    pub fn update(&mut self, data: Option<String>) {
        self.0 = data;
    }

    pub fn get(&self) -> &Option<String> {
        return &self.0;
    }

    pub fn is_present(&self) -> bool {
        return self.0.is_some();
    }
}

impl From<&Artwork> for Artwork {
    fn from(value: &Artwork) -> Self {
        Artwork(value.get().clone())
    }
}

#[derive(Default)]
pub struct ArtworkCache {
    pub id: String,
    pub artwork: Artwork,
}

impl ArtworkCache {
    pub fn mut_read(&mut self, id: &String, playback_service: &impl ArtworkFetcher) -> &Artwork {
        if id.cmp(&self.id).is_ne() {
            self.update_cache(id.clone(), playback_service.get_artwork_string());
        }

        return &self.artwork;
    }

    fn update_cache(&mut self, new_id: String, artwork_string: Option<String>) {
        self.id = new_id;
        self.artwork.update(artwork_string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artwork_cache() {
        let mut cache = ArtworkCache::default();
        let mut artwork_fetcher = MockArtworkFetcher::new();

        artwork_fetcher
            .expect_get_artwork_string()
            .returning(|| Some("artwork1".to_string()));

        let result = cache.mut_read(&"key1".into(), &artwork_fetcher);
        assert_eq!(result.0.as_ref().unwrap(), "artwork1");

        let mut artwork_fetcher = MockArtworkFetcher::new();
        artwork_fetcher
            .expect_get_artwork_string()
            .returning(|| Some("artwork2".to_string()));

        let result = cache.mut_read(&"key1".into(), &artwork_fetcher);
        assert_eq!(result.0.as_ref().unwrap(), "artwork1");

        let result = cache.mut_read(&"key2".into(), &artwork_fetcher);
        assert_eq!(result.0.as_ref().unwrap(), "artwork2");
    }
}
