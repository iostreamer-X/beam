use colored::Colorize;
use dotenv::dotenv;

use beam::{
    artwork_cache::ArtworkCache, consumer::subscibe_and_push_events_to_db, media_cache::MediaCache,
    media_event::MediaEvent, producers::music_media_producer::relay_media_events,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!(
        "{}",
        "
	    ██████╗ ███████╗ █████╗ ███╗   ███╗
	    ██╔══██╗██╔════╝██╔══██╗████╗ ████║
	    ██████╔╝█████╗  ███████║██╔████╔██║
	    ██╔══██╗██╔══╝  ██╔══██║██║╚██╔╝██║
	    ██████╔╝███████╗██║  ██║██║ ╚═╝ ██║
	    ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝
		"
        .green()
    );

    let media_cache = MediaCache::init();
    let artwork_cache = ArtworkCache::init();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<MediaEvent>();

    tokio::spawn(subscibe_and_push_events_to_db(
        rx,
        artwork_cache,
        media_cache,
    ));
    relay_media_events(tx);
}
