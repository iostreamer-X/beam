use serde::Serialize;
use std::fmt::Debug;

use super::music_media::MusicMedia;

pub trait Media: Serialize + Debug {
    fn get_id(&self) -> &String;
    fn get_is_playing(&self) -> bool;
}

impl Media for MusicMedia {
    fn get_id(&self) -> &String {
        &self.name
    }

    fn get_is_playing(&self) -> bool {
        self.is_playing
    }
}
