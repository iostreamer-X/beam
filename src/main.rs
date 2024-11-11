use chrono::Utc;
use dotenv::dotenv;
use std::{ffi::c_void, ptr, sync::Arc};

use core_foundation::{
    base::TCFType, dictionary::CFDictionaryRef, runloop::CFRunLoopRun, string::CFString,
};
use core_foundation_sys::notification_center::{
    CFNotificationCenterAddObserver, CFNotificationCenterGetDistributedCenter,
    CFNotificationCenterRef, CFNotificationName,
    CFNotificationSuspensionBehaviorDeliverImmediately,
};
use currently_playing_uploader::{
    now_playing, voidp_to_ref, ArtworkCache, GenericMedia, GenericMediaObservable,
    GenericMediaStore, Media, MediaEvent, MusicMedia,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex, MutexGuard, OnceCell,
};

static DB: OnceCell<Pool<Postgres>> = OnceCell::const_new();
async fn db() -> &'static Pool<Postgres> {
    DB.get_or_init(|| async {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url.as_str())
            .await
            .expect("Can not create pg pool!")
    })
    .await
}

// There are two source of events: The macOS DistributedNotificationCenter(event driven) & a cli that we poll.
// We use channels to communicate, and each source gets a different one.
// Once we receive an event on a channel, we tidy it up and insert in db.
#[tokio::main]
async fn main() {
    dotenv().ok();

    let aw_cache = Arc::new(Mutex::new(ArtworkCache::default()));
    let generic_media_store = GenericMediaStore::new();

    let (music_tx, music_rx) = tokio::sync::mpsc::unbounded_channel::<MusicMedia>();
    let (generic_tx, generic_rx) = tokio::sync::mpsc::unbounded_channel::<GenericMedia>();

    tokio::spawn(subscribe_to_music_events(music_rx, aw_cache.clone()));
    tokio::spawn(subscribe_to_generic_events(
        generic_media_store,
        generic_rx,
        aw_cache,
    ));

    //Starting polling for local events
    tokio::spawn(GenericMediaObservable::poll(generic_tx));

    //Registering handler for DistributedNotificationCenter and kicking off the run loop
    unsafe {
        let nc = CFNotificationCenterGetDistributedCenter();

        CFNotificationCenterAddObserver(
            nc,
            ptr::addr_of!(music_tx) as *const _, // The transmitter for channel is passed directly to the handler
            music_event_handler,
            CFString::new("com.apple.Music.playerInfo").as_concrete_TypeRef(),
            ptr::null(),
            CFNotificationSuspensionBehaviorDeliverImmediately,
        );
        CFRunLoopRun();
    }
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
        let tx_ref: &UnboundedSender<MusicMedia> = voidp_to_ref(tx_pointer);
        if let Err(e) = tx_ref.send(event) {
            eprintln!("Sending failed to channel! {}", e)
        }
    }
}

// This dangerous method is used in cases when music is stopped and momentarily
// the cli can not fetch artwork but after a few milliseconds it can.
// DANGER: If this takes time then processing time of events can be impacted.
fn force_artwork_cache_to_retrieve(
    mut aw_cache: MutexGuard<'_, ArtworkCache>,
    id: &String,
) -> String {
    if let Some(artwork) = aw_cache.mut_read(id) {
        return artwork.clone();
    }

    return force_artwork_cache_to_retrieve(aw_cache, id);
}

async fn subscribe_to_music_events(
    mut rx: UnboundedReceiver<MusicMedia>,
    aw_cache: Arc<Mutex<ArtworkCache>>,
) {
    while let Some(event) = rx.recv().await {
        let aw_cache = aw_cache.lock().await;
        if !now_playing::is_music() {
            continue;
        }
        let artwork = Some(force_artwork_cache_to_retrieve(aw_cache, event.get_id()));
        let media_event = MediaEvent::Music {
            media: event.clone(),
            artwork,
            emitted_at: Utc::now().timestamp_millis(),
        };
        tokio::spawn(on_media_event_emitted(media_event));
    }
}

async fn subscribe_to_generic_events(
    mut generic_media_store: GenericMediaStore,
    mut rx: UnboundedReceiver<GenericMedia>,
    aw_cache: Arc<Mutex<ArtworkCache>>,
) {
    while let Some(event) = rx.recv().await {
        if let Some(updated_event) = generic_media_store.update(event) {
            let mut aw_cache = aw_cache.lock().await;
            let artwork = aw_cache.mut_read(updated_event.get_id());
            let media_event = MediaEvent::Generic {
                media: updated_event.clone(),
                artwork: artwork.clone(),
                emitted_at: Utc::now().timestamp_millis(),
            };
            tokio::spawn(on_media_event_emitted(media_event));
        }
    }
}

async fn on_media_event_emitted(media: MediaEvent) {
    let db = db().await;
    sqlx::query!(
        "insert into playing(type, data) values ('music', $1) ON CONFLICT (type) DO UPDATE SET data = $1",
        json!(media),
    )
    .execute(db)
    .await
    .expect("Unable to store data to db!");

    println!("Synced successfully!")
}
