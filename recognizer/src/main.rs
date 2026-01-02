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
use crate::recognizer::declarations::{FILE_NOT_FOUND, NO_SONG_MATCH_ERROR};
// use crate::spotify::spotify_utils;

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

fn main() -> Result<(), u8> {
    let args = Args::parse();

    if let Some(add_song_file) = args.add_song {

        let song_title = get_song_title(&add_song_file)?;

        let song_artist = "Ryan Beatty".to_string();

        let song_id = db_utils::store_song(&song_title, &song_artist, "".to_owned())?;

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

        // let best_match: Match = matches[0].clone();

        for matched_song in matches {
            println!("{:?}", matched_song);
        }

        // spotify_utils::track_on_spotify(best_match.song_title, best_match.song_artist)?
    }

    Ok(())
}

fn get_song_title(file_path: &String) -> Result<String, u8> {

    match File::open(file_path) {
        Err(_) => {
            eprintln!("Cannot open file: `{}`", file_path);
            return Err(FILE_NOT_FOUND)
        },
        Ok(_file) => {}
    }

    let file_as_path = Path::new(file_path);

    match file_as_path.file_stem() {
        None => {
            eprintln!("No file stem found: `{}`", file_path);
            Err(FILE_NOT_FOUND)
        },
        Some(file_name) => {
            Ok(file_name.to_str().unwrap().to_string())
        }
    }

}
