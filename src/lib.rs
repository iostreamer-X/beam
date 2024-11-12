use std::{ffi::c_void, fmt::Debug, time::Duration};

use chrono::Utc;
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
        let url =
            Url::parse(store_url.as_str()).expect("[error] Could not parse music media store url!");
        let song_identifier = url
            .query_pairs()
            .find(|(key, _)| key == "i")
            .map(|(_, value)| value)
            .expect("[error] Store url did not have song identifier!");
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
            .expect("[error] Unable to parse if currently playing")
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
    pub async fn poll(tx: UnboundedSender<MediaEvent>) {
        let mut state: Option<(bool, String)> = None;
        loop {
            if let Some(event) = GenericMedia::from_cli() {
                if Self::get_if_state_changed(&state, &event) {
                    state = Some((event.is_playing, event.get_id().clone()));
                    let media_event = MediaEvent::Generic {
                        media: event.clone(),
                        emitted_at: Utc::now().timestamp_millis(),
                    };
                    tx.send(media_event)
                        .expect("[error] Could not send genereic media event!");
                }
            };
            sleep(Duration::from_secs(2)).await;
        }
    }

    fn get_if_state_changed(state: &Option<(bool, String)>, event: &GenericMedia) -> bool {
        if let Some(state) = state {
            let playback_state_changed = state.0 != event.is_playing;
            let media_changed = state.1.cmp(event.get_id()).is_ne();

            return playback_state_changed || media_changed;
        }

        return true;
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
        emitted_at: i64,
    },
    Generic {
        media: GenericMedia,
        emitted_at: i64,
    },
}

impl MediaEvent {
    pub fn get_id(&self) -> &String {
        match self {
            MediaEvent::Music {
                media,
                emitted_at: _,
            } => media.get_id(),
            MediaEvent::Generic {
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
            MediaEvent::Generic {
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
            MediaEvent::Generic {
                media: _,
                emitted_at: _,
            } => "generic",
        }
    }

    pub fn get_emitted_at(&self) -> &i64 {
        match self {
            MediaEvent::Music {
                media: _,
                emitted_at,
            } => emitted_at,
            MediaEvent::Generic {
                media: _,
                emitted_at,
            } => emitted_at,
        }
    }
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
    pub fn mut_read(&mut self, id: &String) -> &Artwork {
        if id.cmp(&self.id).is_ne() {
            self.artwork.update(now_playing::get_artwork_string());
        }

        return &self.artwork;
    }
}

pub unsafe fn voidp_to_ref<'a, T>(p: *const c_void) -> &'a T {
    unsafe { &*(p as *const T) }
}
