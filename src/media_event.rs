use serde::Serialize;

use crate::medias::{media::Media, music_media::MusicMedia};

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum MediaEvent {
    Music { media: MusicMedia, emitted_at: i64 },
}

impl MediaEvent {
    pub fn get_id(&self) -> &String {
        match self {
            MediaEvent::Music {
                media,
                emitted_at: _,
            } => media.get_id(),
        }
    }
    pub fn get_is_playing(&self) -> bool {
        match self {
            MediaEvent::Music {
                media,
                emitted_at: _,
            } => media.get_is_playing(),
        }
    }
    pub fn get_type(&self) -> &'static str {
        match self {
            MediaEvent::Music {
                media: _,
                emitted_at: _,
            } => "music",
        }
    }

    pub fn get_emitted_at(&self) -> &i64 {
        match self {
            MediaEvent::Music {
                media: _,
                emitted_at,
            } => emitted_at,
        }
    }
}
