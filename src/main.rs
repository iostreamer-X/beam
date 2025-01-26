use chrono::Utc;
use colored::Colorize;
use dotenv::dotenv;
use std::{ffi::c_void, ptr};

use beam::{
    artwork::ArtworkCache, now_playing::NowPlayingService, voidp_to_ref, GenericMediaObservable,
    Media, MediaEvent, MusicMedia,
};
use core_foundation::{
    base::TCFType, dictionary::CFDictionaryRef, runloop::CFRunLoopRun, string::CFString,
};
use core_foundation_sys::notification_center::{
    CFNotificationCenterAddObserver, CFNotificationCenterGetDistributedCenter,
    CFNotificationCenterRef, CFNotificationName,
    CFNotificationSuspensionBehaviorDeliverImmediately,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    OnceCell,
};

static DB: OnceCell<Pool<Postgres>> = OnceCell::const_new();
async fn db() -> &'static Pool<Postgres> {
    DB.get_or_init(|| async {
        let db_url = std::env::var("BEAM_DATABASE_URL").expect("[error] DATABASE_URL must be set.");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url.as_str())
            .await
            .expect("[error] Can not create pg pool!")
    })
    .await
}

extern "C" fn music_event_handler(
    _: CFNotificationCenterRef,
    tx_pointer: *mut c_void,
    _: CFNotificationName,
    _: *const c_void,
    user_info: CFDictionaryRef,
) {
    unsafe {
        let event = MusicMedia::from_cf_dictionary(user_info);
        let tx_ref: &UnboundedSender<MediaEvent> = voidp_to_ref(tx_pointer);
        let cli_output = NowPlayingService::parse_cli_raw();
        let now_playing = match NowPlayingService::get_now_playing(&cli_output) {
            Ok(result) => result,
            _ => return,
        };
        if !now_playing.is_music {
            return;
        }
        let media_event = MediaEvent::Music {
            media: event.clone(),
            emitted_at: Utc::now().timestamp_millis(),
        };
        if let Err(e) = tx_ref.send(media_event) {
            println!("{} Sending failed to channel! {}", "[error]".red(), e)
        }
    }
}

async fn subscribe_to_media_events(
    mut rx: UnboundedReceiver<MediaEvent>,
    mut generic_media_aw_cache: ArtworkCache,
    mut music_media_aw_cache: ArtworkCache,
) {
    let db = db().await;
    println!("{} Database connection initialized", "[log]".blue());
    while let Some(event) = rx.recv().await {
        let artwork_result = match &event {
            MediaEvent::Generic {
                media,
                emitted_at: _,
            } => generic_media_aw_cache.mut_read(media.get_id()),
            MediaEvent::Music {
                media,
                emitted_at: _,
            } => music_media_aw_cache.mut_read(media.get_id()),
        };

        if let Some(artwork) = artwork_result {
            let mut raw_event = json!(event);
            let raw_event = raw_event.as_object_mut().unwrap();
            raw_event.insert("artwork".into(), json!(artwork));
            insert_into_db(json!(raw_event), db).await;
            println!(
                "{} type='{}' name='{}' state={} latency_ms={}",
                "[synced]".green(),
                event.get_type(),
                event.get_id(),
                event.get_is_playing(),
                chrono::Utc::now().timestamp_millis() - event.get_emitted_at()
            );
        }
    }
}

async fn insert_into_db(record: serde_json::Value, db: &Pool<Postgres>) {
    sqlx::query!(
            "insert into playing(type, data) values ('music', $1) ON CONFLICT (type) DO UPDATE SET data = $1",
            json!(record),
        )
        .execute(db)
        .await
        .expect("[error] Unable to store data to db!");
}

// There are two source of events: The macOS DistributedNotificationCenter(event driven) & a cli that we poll.
// We use channels to communicate, and each source gets a different one.
// Once we receive an event on a channel, we tidy it up and insert in db.
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

    let generic_media_aw_cache = ArtworkCache::default();
    let music_media_aw_cache = ArtworkCache::default();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<MediaEvent>();

    tokio::spawn(subscribe_to_media_events(
        rx,
        generic_media_aw_cache,
        music_media_aw_cache,
    ));

    //Starting polling for local events
    tokio::spawn(GenericMediaObservable::poll(tx.clone()));

    //Registering handler for DistributedNotificationCenter and kicking off the run loop
    unsafe {
        let nc = CFNotificationCenterGetDistributedCenter();

        CFNotificationCenterAddObserver(
            nc,
            ptr::addr_of!(tx) as *const _, // The transmitter for channel is passed directly to the handler
            music_event_handler,
            CFString::new("com.apple.Music.playerInfo").as_concrete_TypeRef(),
            ptr::null(),
            CFNotificationSuspensionBehaviorDeliverImmediately,
        );
        CFRunLoopRun();
    }
}
