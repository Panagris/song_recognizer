/*
file: /src/db/db.rs
provides common functions to interact with the database of songs
*/
use crate::db::models::{Fingerprint, NewFingerprint, NewSong, Song};
use crate::recognizer::declarations::{
    DATABASE_INSERT_ERROR, DATABASE_QUERY_ERROR, NO_SONG_MATCH_ERROR,
};
use crate::recognizer::fingerprint::KeyAudioPoint;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::collections::HashMap;
use std::env;

pub fn store_fingerprints(fingerprint_map: HashMap<u32, KeyAudioPoint>) -> Result<(), u8> {
    use crate::db::schema::fingerprints;

    let connection = &mut establish_connection();

    for (hash, point) in fingerprint_map {
        let (anchor_time_ms, song_id) = (point.anchor_time_ms, point.song_id);

        let new_fingerprint = NewFingerprint {
            hash: hash as i32,
            anchor_time_ms,
            song_id,
        };

        match diesel::insert_into(fingerprints::table)
            .values(&new_fingerprint)
            .execute(connection)
        {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error saving new fingerprint");
                return Err(DATABASE_INSERT_ERROR);
            }
        }
    }
    Ok(())
}

pub fn get_key_audio_points(hashes: Vec<i32>) -> Result<HashMap<u32, Vec<KeyAudioPoint>>, u8> {
    use crate::db::schema::fingerprints;
    let mut key_audio_points = HashMap::<u32, Vec<KeyAudioPoint>>::new();
    let connection = &mut establish_connection();

    for hash in hashes {
        let matching_points: Vec<Fingerprint> = match fingerprints::table
            .filter(fingerprints::hash.eq(hash))
            .load::<Fingerprint>(connection)
        {
            Ok(v) => v,
            Err(_) => return Err(DATABASE_QUERY_ERROR),
        };

        let points_from_db = matching_points
            .into_iter()
            .map(|fingerprint: Fingerprint| KeyAudioPoint {
                anchor_time_ms: fingerprint.anchor_time_ms,
                song_id: fingerprint.song_id,
            })
            .collect::<Vec<KeyAudioPoint>>();

        key_audio_points.insert(hash as u32, points_from_db);
    }

    Ok(key_audio_points)
}

pub fn store_song(title: &String, artist: &String, spotify_uri: Option<String>) -> Result<u32, u8> {
    use crate::db::schema::songs;

    let connection = &mut establish_connection();

    let song_key: String = title.to_owned() + "---" + &artist;

    let new_post = NewSong {
        title: title.to_owned(),
        artist: artist.to_owned(),
        spotify_uri,
        song_key,
    };

    let song: Song = match diesel::insert_into(songs::table)
        .values(&new_post)
        .returning(Song::as_returning())
        .get_result(connection)
    {
        Ok(inserted_song) => inserted_song,
        Err(_) => {
            eprintln!("Error saving song to database!");
            return Err(DATABASE_INSERT_ERROR);
        }
    };

    Ok(song.id as u32)
}

pub fn get_song_by_id(song_id: u32) -> Result<Song, u8> {
    use crate::db::schema::songs;

    let connection = &mut establish_connection();

    let matching_songs: Vec<Song> = match songs::table
        .filter(songs::id.eq(song_id as i32))
        .load::<Song>(connection)
    {
        Ok(songs) => songs,
        Err(_) => return Err(DATABASE_QUERY_ERROR),
    };

    if matching_songs.len() < 1 {
        return Err(NO_SONG_MATCH_ERROR);
    }

    Ok(matching_songs[0].clone())
}

// pub fn get_song_by_song_key(song_key: String) -> Result<Song, u8> {
//     use crate::db::schema::songs;
//
//     let connection = &mut establish_connection();
//
//     let matching_songs: Vec<Song> = match songs::table
//         .filter(songs::song_key.eq(song_key))
//         .load::<Song>(connection)
//     {
//         Ok(songs) => songs,
//         Err(_) => return Err(DATABASE_QUERY_ERROR),
//     };
//
//     if matching_songs.len() < 1 {
//         return Err(NO_SONG_MATCH_ERROR);
//     }
//
//     Ok(matching_songs[0].clone())
// }

pub(crate) fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    SqliteConnection::establish(database_url.as_str())
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Using a song_title and song_artist, finds the corresponding DB entry and udpates the uri
pub(crate) fn update_song_uri(
    song_title: &String,
    song_artist: &String,
    uri: String,
) -> Result<(), u8> {

    use crate::db::schema::songs;

    let connection = &mut establish_connection();

    let song_key: String = song_title.to_owned() + "---" + song_artist;

    match diesel::update(songs::table)
        .filter(songs::song_key.eq(song_key))
        .set(songs::spotify_uri.eq(uri))
        .execute(connection)
    {
        Ok(_) => Ok(()),
        Err(_) => {
            eprintln!("Error updating `{}` Spotify uri!", song_title);
            Err(DATABASE_INSERT_ERROR)
        }
    }
}
