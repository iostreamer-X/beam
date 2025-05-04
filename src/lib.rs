use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::ffi::c_void;
use tokio::sync::OnceCell;

pub mod artwork;
pub mod artwork_cache;
pub mod consumer;
pub mod media_cache;
pub mod media_event;
pub mod medias;
pub mod producers;

static DB: OnceCell<Pool<Postgres>> = OnceCell::const_new();
pub async fn db() -> &'static Pool<Postgres> {
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

pub async fn insert_into_db(record: serde_json::Value, db: &Pool<Postgres>) {
    sqlx::query!(
            "insert into playing(type, data) values ('music', $1) ON CONFLICT (type) DO UPDATE SET data = $1",
            json!(record),
        )
        .execute(db)
        .await
        .expect("[error] Unable to store data to db!");
}

pub unsafe fn voidp_to_ref<'a, T>(p: *const c_void) -> &'a T {
    unsafe { &*(p as *const T) }
}
