// file: recognizer/src/db/models.rs
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::fingerprints)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Fingerprint {
    #[allow(unused)]
    pub hash: i32,
    pub anchor_time_ms: i32,
    pub song_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::db::schema::fingerprints)]
pub struct NewFingerprint {
    pub hash: i32,
    pub anchor_time_ms: i32,
    pub song_id: i32,
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::db::schema::songs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Song {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub spotify_uri: Option<String>,
    #[allow(unused)]
    pub song_key: String // form: song--artist, for easy non-ID lookup
}

#[derive(Insertable)]
#[diesel(table_name = crate::db::schema::songs)]
pub struct NewSong {
    pub title: String,
    pub artist: String,
    pub spotify_uri: Option<String>,
    pub song_key: String
}