use std::{ffi::c_void, fmt::Debug, time::Duration};

use core_foundation::{
    base::{FromVoid, TCFType},
    dictionary::{CFDictionaryGetValue, CFDictionaryRef},
    string::CFString,
};
use serde::Serialize;
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use url::Url;
pub mod now_playing;

pub trait Media: Serialize + Debug {
    fn get_id(&self) -> &String;
    fn get_is_playing(&self) -> bool;
}

#[derive(Debug, Serialize, Clone)]
pub struct MusicMedia {
    is_playing: bool,
    genre: String,
    album: String,
    artist: String,
    name: String,
    link: String,
}

impl MusicMedia {
    // This parses the event received from macOS DistributedNotificationCenter
    pub unsafe fn from_cf_dictionary(dictionary: CFDictionaryRef) -> Self {
        let is_playing =
            Self::get_string_key_from_cf_dictionary(dictionary, "Player State").eq("Playing");
        let store_url = Self::get_string_key_from_cf_dictionary(dictionary, "Store URL");
        let url = Url::parse(store_url.as_str()).expect("Could not parse music media store url!");
        let song_identifier = url
            .query_pairs()
            .find(|(key, _)| key == "i")
            .map(|(_, value)| value)
            .expect("Store url did not have song identifier!");
        let link = format!("https://music.apple.com/us/song/{}", song_identifier);
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

#[derive(Debug, Serialize, Clone)]
pub struct GenericMedia {
    is_playing: bool,
    artist: Option<String>,
    name: String,
}

impl GenericMedia {
    fn parse_is_playing_from_cli() -> bool {
        let result = now_playing::parse_cli("PlaybackRate");
        result
            .parse::<u8>()
            .expect("Unable to parse if currently playing")
            == 1
    }
    pub fn from_cli() -> Option<Self> {
        if !now_playing::is_music() {
            Some(Self {
                is_playing: Self::parse_is_playing_from_cli(),
                artist: now_playing::parse_cli_optional("Artist"),
                name: now_playing::parse_cli("Title"),
            })
        } else {
            None
        }
    }
}

pub struct GenericMediaObservable;
impl GenericMediaObservable {
    pub async fn poll(tx: UnboundedSender<GenericMedia>) {
        loop {
            if let Some(event) = GenericMedia::from_cli() {
                tx.send(event)
                    .expect("Could not send genereic media event!");
            };
            sleep(Duration::from_secs(2)).await;
        }
    }
}

impl Media for MusicMedia {
    fn get_id(&self) -> &String {
        &self.name
    }

    fn get_is_playing(&self) -> bool {
        self.is_playing
    }
}

impl Media for GenericMedia {
    fn get_id(&self) -> &String {
        &self.name
    }

    fn get_is_playing(&self) -> bool {
        self.is_playing
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum MediaEvent {
    Music {
        media: MusicMedia,
        artwork: Option<String>,
        emitted_at: i64,
    },
    Generic {
        media: GenericMedia,
        artwork: Option<String>,
        emitted_at: i64,
    },
}

#[derive(Default)]
pub struct ArtworkCache {
    pub id: String,
    pub artwork: Option<String>,
}

impl ArtworkCache {
    pub fn mut_read(&mut self, id: &String) -> &Option<String> {
        if id.cmp(&self.id).is_eq() {
            return &self.artwork;
        }
        self.artwork = now_playing::get_artwork();
        return &self.artwork;
    }
}

pub struct GenericMediaStore {
    event: Option<GenericMedia>,
}

impl GenericMediaStore {
    pub fn new() -> Self {
        Self {
            event: GenericMedia::from_cli(),
        }
    }

    pub fn update(&mut self, other_event: GenericMedia) -> &Option<GenericMedia> {
        let playback_state_changed = self.event.as_ref().map_or_else(
            || true,
            |state| state.get_is_playing() != other_event.get_is_playing(),
        );
        let song_changed = self
            .event
            .as_ref()
            .map_or_else(|| true, |state| state.get_id() != other_event.get_id());

        if playback_state_changed || song_changed {
            self.event.replace(other_event);
            return &self.event;
        }

        return &None;
    }

    pub fn print_event(&self) {
        println!("Generic Media Event{:?}", self.event)
    }
}

pub unsafe fn voidp_to_ref<'a, T>(p: *const c_void) -> &'a T {
    unsafe { &*(p as *const T) }
}
