use std::process::Command;

pub fn get_artwork_string() -> Option<String> {
    parse_cli_optional("ArtworkData")
}

pub fn get_song_identifier() -> String {
    parse_cli("ContentItemIdentifier")
}

pub fn is_music() -> bool {
    parse_cli_optional("MediaType").map_or_else(
        || false,
        |media_type| media_type == "MRMediaRemoteMediaTypeMusic",
    )
}

pub fn parse_cli(arg: &str) -> String {
    let output = Command::new("nowplaying-cli")
        .args(["get", arg])
        .output()
        .expect("failed to execute process");

    String::from_utf8(output.stdout)
        .expect("Command output not a valid utf-8 string!")
        .trim()
        .to_string()
}

pub fn parse_cli_optional(arg: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use crate::now_playing::parse_cli;

    #[test]
    fn parse_song_identifier() {
        let result = parse_cli("ContentItemIdentifier");
        assert!(result.len() > 4);
    }

    #[test]
    fn parse_album_identifier() {
        let result = parse_cli("AlbumiTunesStoreAdamIdentifier");
        assert!(result.len() > 4);
    }
}
