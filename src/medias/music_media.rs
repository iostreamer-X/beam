use serde::Serialize;

use core_foundation::{
    base::{FromVoid, TCFType},
    dictionary::{CFDictionaryGetValue, CFDictionaryRef},
    string::CFString,
};

use url::Url;

#[derive(Debug, Serialize, Clone)]
pub struct MusicMedia {
    pub is_playing: bool,
    genre: String,
    album: String,
    artist: String,
    pub name: String,
    link: Option<String>,
}

impl MusicMedia {
    // This parses the event received from macOS DistributedNotificationCenter
    pub unsafe fn from_cf_dictionary(dictionary: CFDictionaryRef) -> Self {
        let is_playing =
            Self::get_string_key_from_cf_dictionary(dictionary, "Player State").eq("Playing");
        let store_url = Self::get_string_key_from_cf_dictionary(dictionary, "Store URL");
        let url =
            Url::parse(store_url.as_str()).expect("[error] Could not parse music media store url!");
        let link = url
            .query_pairs()
            .find(|(key, _)| key == "i")
            .map(|(_, song_identifier)| {
                format!("https://music.apple.com/us/song/{}", song_identifier)
            });
        Self {
            is_playing,
            genre: Self::get_string_key_from_cf_dictionary(dictionary, "Genre"),
            album: Self::get_string_key_from_cf_dictionary(dictionary, "Album"),
            artist: Self::get_string_key_from_cf_dictionary(dictionary, "Artist"),
            name: Self::get_string_key_from_cf_dictionary(dictionary, "Name"),
            link,
        }
    }

    unsafe fn get_string_key_from_cf_dictionary(dictionary: CFDictionaryRef, key: &str) -> String {
        let raw_value = CFDictionaryGetValue(dictionary, CFString::new(key).as_CFTypeRef());
        let cf_string = CFString::from_void(raw_value);
        cf_string.to_string()
    }
}
