use std::{ffi::c_void, fmt::Debug, time::Duration};

use chrono::Utc;
use core_foundation::{
    base::{FromVoid, TCFType},
    dictionary::{CFDictionaryGetValue, CFDictionaryRef},
    string::CFString,
};
use now_playing::NowPlayingService;
use serde::Serialize;
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use url::Url;

pub mod artwork;
pub mod now_playing;
pub mod now_playing_raw_parser;

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

impl Media for GenericMedia {
    fn get_id(&self) -> &String {
        &self.name
    }

    fn get_is_playing(&self) -> bool {
        self.is_playing
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct MusicMedia {
    is_playing: bool,
    genre: String,
    album: String,
    artist: String,
    name: String,
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

#[derive(Debug, Serialize, Clone)]
pub struct GenericMedia {
    is_playing: bool,
    artist: Option<String>,
    name: String,
}

// While MusicMedia is event driven, GenericMedia is polled from CLI.
// The CLI captures every other media being played on the machine, including MusicMedia(iTunes/Music app).
// This means GenericMedia filters on the basis of it being MusicMedia or not, so as to not send a repeat event.
impl GenericMedia {
    pub fn from_cli<'a>(output: &'a String) -> Option<Self> {
        let now_playing = match NowPlayingService::get_now_playing(output) {
            Ok(result) => result,
            _ => return None,
        };

        if now_playing.is_music {
            None
        } else {
            Some(Self {
                is_playing: now_playing.is_playing,
                artist: now_playing.artist.map(|v| v.to_string()),
                name: now_playing.title.to_string(),
            })
        }
    }
}

pub struct GenericMediaObservable;
impl GenericMediaObservable {
    pub async fn poll(tx: UnboundedSender<MediaEvent>) {
        let mut state: Option<GenericMedia> = None;
        loop {
            let cli_output = NowPlayingService::parse_cli_raw();
            if let Some(event) = GenericMedia::from_cli(&cli_output) {
                if Self::get_if_state_changed(&state, &event) {
                    state = Some(event.clone());
                    let media_event = MediaEvent::Generic {
                        media: event.clone(),
                        emitted_at: Utc::now().timestamp_millis(),
                    };
                    tx.send(media_event)
                        .expect("[error] Could not send genereic media event!");
                }
            } else {
                // In case where before playing GenericMedia(youtube on browser), MusicMedia(iTunes/Music app) was being played,
                // and then we pause the GenericMedia, the CLI falls back to MusicMedia.
                // And as you might recall, GenericMedia can not be constructed if CLI tells MusicMedia is/was being played.
                // This leads to a behaviour where no 'Pause' or 'Stopped' event is fired for GenericMedia.
                // In the following block, we are emulating that behaviour.
                if let Some(override_event) = Self::get_override_event(&state) {
                    state = Some(override_event.clone());
                    let media_event = MediaEvent::Generic {
                        media: override_event,
                        emitted_at: Utc::now().timestamp_millis(),
                    };
                    tx.send(media_event)
                        .expect("[error] Could not send genereic media event!");
                }
            };
            sleep(Duration::from_secs(2)).await;
        }
    }

    fn get_override_event(previous_state: &Option<GenericMedia>) -> Option<GenericMedia> {
        if let Some(previous_state) = previous_state {
            let cli_output = NowPlayingService::parse_cli_raw();
            let is_playing = match NowPlayingService::get_now_playing(&cli_output) {
                Ok(result) => result.is_playing,
                _ => false,
            };
            if previous_state.get_is_playing() && !is_playing {
                return Some(GenericMedia {
                    is_playing: false,
                    ..previous_state.clone()
                });
            }
            return None;
        } else {
            None
        }
    }

    fn get_if_state_changed(state: &Option<GenericMedia>, event: &GenericMedia) -> bool {
        if let Some(state) = state {
            let playback_state_changed = state.is_playing != event.is_playing;
            let media_changed = event.name.cmp(&state.name).is_ne();

            return playback_state_changed || media_changed;
        }

        return true;
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
            } => &media.name,
            MediaEvent::Generic {
                media,
                emitted_at: _,
            } => &media.name,
        }
    }
    pub fn get_is_playing(&self) -> bool {
        match self {
            MediaEvent::Music {
                media,
                emitted_at: _,
            } => media.is_playing,
            MediaEvent::Generic {
                media,
                emitted_at: _,
            } => media.is_playing,
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

pub unsafe fn voidp_to_ref<'a, T>(p: *const c_void) -> &'a T {
    unsafe { &*(p as *const T) }
}
