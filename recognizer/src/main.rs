/* file: src/main.rs

*/

mod db;
mod recognizer;
mod spotify;

use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use crate::recognizer::fingerprint;
use crate::recognizer::fingerprint::KeyAudioPoint;
use crate::recognizer::shazam;
use crate::recognizer::shazam::Match;
use crate::db::db_utils;
use crate::recognizer::declarations::{FILE_NOT_FOUND, INCOMPATIBLE_FILE_ERROR, NO_SONG_MATCH_ERROR};
use crate::spotify::spotify_utils;

/** A program to identify songs from an audio file.
Functions similar to Shazam.
Also allows users to add personal songs to a database.
*/
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Audio file [.wav] to add to database
    #[arg(short, long)]
    add_song: Option<String>,

    /// Audio file to recognize song
    #[arg(short, long)]
    id_song: Option<String>,
    // TODO: add option to listen live from microphone
}

#[tokio::main]
async fn main() -> Result<(), u8> {

    let _file = "../songs/White Teeth_Ryan Beatty.wav".to_string();
    let _song: String = "White Teeth".to_string();
    let _artist: String = "Ryan Beatty".to_string();

    assert_eq!( get_song_title_artist(&_file), Ok((_song, _artist)) );

    let args = Args::parse();

    if let Some(add_song_file) = args.add_song {

        let (song_title, song_artist) = get_song_title_artist(&add_song_file)?;

        let uri = spotify_utils::get_track_uri(&song_title, &song_artist).await;

        let song_id = db_utils::store_song(&song_title, &song_artist, uri)?;

        let fingerprint = fingerprint::fingerprint_audio(&add_song_file, song_id)?;

        db_utils::store_fingerprints(fingerprint)?;
    }

    if let Some(id_song_file) = args.id_song {
        let song_id = rand::random::<u32>();

        let sample_fingerprint_map: HashMap<u32, KeyAudioPoint> =
            fingerprint::fingerprint_audio(&id_song_file, song_id)?;

        let fingerprint: HashMap<u32, u32> =
            sample_fingerprint_map.into_iter().map( |(hash, key_audio_point)| {
                (hash, key_audio_point.anchor_time_ms as u32)
            }).collect();

        let matches: Vec<Match> = shazam::find_matches_from_fingerprint(fingerprint)?;

        if matches.len() < 1 {
            eprintln!("No matches found for `{}`!", id_song_file);
            return Err(NO_SONG_MATCH_ERROR);
        }

        let best_match: Match = matches[0].clone();

        if let Some(uri) = best_match.spotify_uri {
            if uri.is_empty() {

                let uri: String = spotify_utils::play_song(
                    &best_match.song_title, &best_match.song_artist
                ).await?;

                db_utils::update_song_uri(&best_match.song_title, &best_match.song_artist, uri)?;
            } else {
                spotify_utils::play_song_from_uri(&uri).await?;
            }

        } else {

            let uri: String = spotify_utils::play_song(
                &best_match.song_title, &best_match.song_artist
            ).await?;

            db_utils::update_song_uri(&best_match.song_title, &best_match.song_artist, uri)?;
        }
    }

    Ok(())
}

fn get_song_title_artist(file_path: &String) -> Result<(String, String), u8> {

    match File::open(file_path) {
        Err(_) => {
            eprintln!("Cannot open file: `{}`", file_path);
            return Err(FILE_NOT_FOUND)
        },
        Ok(_file) => {}
    }

    let file_as_path = Path::new(file_path);

    let file_name: &str = match file_as_path.file_stem() {
        None => {
            eprintln!("No file stem found: `{}`", file_path);
            return Err(FILE_NOT_FOUND);
        },

        Some(file_name) => {
            file_name.to_str().unwrap()
        }
    };

    let vec_names: Vec<&str> = file_name.split('_').collect();

    if vec_names.len() < 2 {
        return Err(INCOMPATIBLE_FILE_ERROR);
    }

    let (song_name, artist_name) = (vec_names[0], vec_names[1]);

    Ok( (song_name.to_string(), artist_name.to_string()) )
}
