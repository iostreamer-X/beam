use crate::NowPlayingService;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Artwork(String);

impl From<&Artwork> for Artwork {
    fn from(value: &Artwork) -> Self {
        Artwork(value.0.clone())
    }
}

#[derive(Default)]
pub struct ArtworkCache {
    pub id: String,
    pub artwork: Option<Artwork>,
}

impl ArtworkCache {
    pub fn mut_read(&mut self, id: &String) -> &Option<Artwork> {
        if id.cmp(&self.id).is_ne() {
            self.update_cache(id.clone());
        }

        return &self.artwork;
    }

    fn update_cache(&mut self, new_id: String) {
        self.id = new_id;

        // In some cases when music stops, the cli isn't able to pick up the artwork.
        // So we loop over till we get it. We do it only if music is being played.
        let artwork_result = loop {
            let artwork = NowPlayingService::get_artwork_string();
            if artwork.is_some() {
                break Ok(artwork.unwrap());
            }
            let cli_output = NowPlayingService::parse_cli_raw();
            let now_playing = match NowPlayingService::get_now_playing(&cli_output) {
                Ok(result) => result,
                _ => break Err("Can not force artwork retrieval if music is not playing!"),
            };
            if !now_playing.is_music {
                break Err("Can not force artwork retrieval if music is not playing!");
            }
        };

        if let Ok(artwork) = artwork_result {
            self.artwork = Some(Artwork(artwork));
        } else if let Err(e) = artwork_result {
            println!("[error] Could not update artwork cache! {}", e);
        }
    }
}
