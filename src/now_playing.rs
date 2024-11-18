use crate::{
    artwork::ArtworkFetcher,
    now_playing_raw_parser::{self, ParsingError},
};
use std::{collections::HashMap, process::Command};

const PLAYBACK_RATE: &str = "kMRMediaRemoteNowPlayingInfoPlaybackRate";
const TITLE: &str = "kMRMediaRemoteNowPlayingInfoTitle";
const ARTIST: &str = "kMRMediaRemoteNowPlayingInfoArtist";
const MEDIA_TYPE: &str = "kMRMediaRemoteNowPlayingInfoMediaType";

#[derive(Debug)]
pub struct NowPlaying<'a> {
    pub is_music: bool,
    pub is_playing: bool,
    pub title: &'a str,
    pub artist: Option<&'a str>,
}

#[derive(Default)]
pub struct NowPlayingService;

impl ArtworkFetcher for NowPlayingService {
    fn get_artwork_string(&self) -> Option<String> {
        Self::parse_cli_optional("ArtworkData")
    }
}

impl NowPlayingService {
    pub fn get_now_playing<'a>(output: &'a String) -> Result<NowPlaying<'a>, ParsingError> {
        let hashmap = Self::get_hashmap(output)?;
        let is_music = hashmap
            .get(MEDIA_TYPE)
            .map_or(false, |value| *value == "MRMediaRemoteMediaTypeMusic");
        let is_playing = hashmap.get(PLAYBACK_RATE).map_or(false, |value| {
            value
                .parse::<u8>()
                .expect("[error] Unable to parse if currently playing")
                == 1
        });
        let title = hashmap
            .get(TITLE)
            .expect("[error] Unable to parse media title from cli");
        let artist = hashmap.get(ARTIST).map(|v| *v);

        Ok(NowPlaying {
            is_music,
            is_playing,
            title,
            artist,
        })
    }

    fn get_hashmap(output: &String) -> Result<HashMap<&str, &str>, ParsingError> {
        now_playing_raw_parser::parse_raw(&output)
    }

    pub fn parse_cli_raw() -> String {
        let output = Command::new("nowplaying-cli")
            .args(["get-raw"])
            .output()
            .expect("failed to execute process");

        String::from_utf8(output.stdout)
            .expect("Command output not a valid utf-8 string!")
            .trim()
            .to_string()
    }

    fn parse_cli_optional(arg: &str) -> Option<String> {
        let output = Command::new("nowplaying-cli")
            .args(["get", arg])
            .output()
            .expect("failed to execute process");

        let result = String::from_utf8(output.stdout)
            .expect("Command output not a valid utf-8 string!")
            .trim()
            .to_string();

        if result.len() == 0 || result == "null" {
            return None;
        }

        return Some(result);
    }
}

#[cfg(test)]
mod tests {
    use super::NowPlayingService;

    #[test]
    fn test_get_now_playing() {
        let cli_output = NowPlayingService::parse_cli_raw();
        NowPlayingService::get_now_playing(&cli_output).expect("Could not parse cli output");
    }
}
