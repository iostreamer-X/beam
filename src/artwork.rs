use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use image::ImageReader;
use std::{io::Cursor, process::Command};

use serde::Serialize;

const ARTWORK_PATH: &'static str = "/tmp/current_artwork.jpg";

#[derive(Debug, Default, Serialize)]
pub struct Artwork(String);

impl Artwork {
    pub fn try_init() -> Result<Artwork> {
        let result = Command::new("osascript")
            .arg("-e")
            .arg(get_script())
            .output()?;
        let result_string = String::from_utf8(result.stdout)?;
        let result = result_string.trim().parse::<i8>()?;
        if result == -1 {
            return Err(anyhow!("Could not fetch artwork from script!"));
        }

        let image = ImageReader::open(ARTWORK_PATH)?.decode()?;
        let mut image_data: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut image_data), image::ImageFormat::Jpeg)
            .unwrap();
        let res_base64 = general_purpose::STANDARD.encode(image_data);
        Result::Ok(Artwork(res_base64))
    }

    pub fn get_string(&self) -> String {
        self.0.clone()
    }
}

fn get_script() -> String {
    return format!(
        "
        set filePath to \"{}\"
        tell application \"Music\"
			if player state is playing then
				set theTrack to current track
				try
					set artData to data of artwork 1 of theTrack
					set outFile to open for access (POSIX file filePath) with write permission
					set eof of outFile to 0 -- clear existing contents
					write artData to outFile
					close access outFile
					return 0
				on error errMsg
					try
						close access outFile
					end try
					return -1
				end try
			else
				return -1
			end if
		end tell
		",
        ARTWORK_PATH
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_base64_image() {
        let result = Artwork::try_init().expect("Could not get Image!");
        println!("{:?}", result);
    }
}
