use anyhow::Result;
use colored::Colorize;
use serde_json::json;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    artwork_cache::ArtworkCache, db, insert_into_db, media_cache::MediaCache,
    media_event::MediaEvent,
};

pub async fn subscibe_and_push_events_to_db(
    mut rx: UnboundedReceiver<MediaEvent>,
    mut artwork_cache: ArtworkCache,
    mut media_cache: MediaCache,
) {
    let db = db().await;
    println!("{} Database connection initialized", "[log]".blue());
    while let Some(event) = rx.recv().await {
        let result = media_event_handler(event, &mut artwork_cache, &mut media_cache, db).await;
        result.map_or_else(
            |e| println!("{} {}", "[error]".red(), e),
            |r| println!("{} {}", "[synced]".green(), r),
        );
    }
}

async fn media_event_handler(
    event: MediaEvent,
    artwork_cache: &mut ArtworkCache,
    media_cache: &mut MediaCache,
    db: &Pool<Postgres>,
) -> Result<String> {
    let did_media_change = media_cache.update(event.get_id());
    if did_media_change {
        artwork_cache.clear();
    }
    let artwork = match &event {
        MediaEvent::Music {
            media: _,
            emitted_at: _,
        } => artwork_cache.get()?,
    };

    let mut raw_event = json!(event);
    let raw_event = raw_event.as_object_mut().unwrap();
    raw_event.insert("artwork".into(), json!(artwork.get_string()));
    insert_into_db(json!(raw_event), db).await;
    return Ok(format!(
        "type='{}' name='{}' state={} latency_ms={}",
        event.get_type(),
        event.get_id(),
        event.get_is_playing(),
        chrono::Utc::now().timestamp_millis() - event.get_emitted_at()
    ));
}
