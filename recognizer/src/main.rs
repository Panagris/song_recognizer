/* file: src/main.rs
Purpose: provides the application's external interface, allowing users to add songs to a database
 for future identification as well as identify a song based on an audio snippet.
*/

mod db;
mod recognizer;
mod spotify;

use crate::db::db_utils;
use crate::recognizer::declarations::{DATABASE_INSERT_ERROR, FILE_NOT_FOUND, INCOMPATIBLE_FILE_ERROR, MATCH_SCORE_THRESHOLD, NO_SONG_MATCH_ERROR};
use crate::recognizer::fingerprint;
use crate::recognizer::fingerprint::KeyAudioPoint;
use crate::recognizer::shazam;
use crate::recognizer::shazam::Match;
use crate::spotify::spotify_utils;
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

// Struct for `clap` crate to handle command-line arguments.
#[derive(Parser, Debug)]
#[command(about = "Compares audio snippet against songs in a database to determine the snippet's \
song title and artist", long_about = None)]
struct Args {
    /// Audio file(s) [.wav] to add to the database. Repeat flag for each additional file
    #[arg(short, long, value_name = "FILE")]
    add_song: Vec<String>,

    // The following three commands are mutually exclusive!
    /// Audio file [.wav] to compare against songs in the database
    #[arg(short, long, value_name = "FILE", group = "input")]
    id_song: Option<String>,

    // TODO: add option to listen live from microphone or interface
    /// Listen on the device's default microphone
    #[arg(long, group = "input")]
    microphone: Option<String>,

    /// Specify a particular interface capable of audio-capture
    #[arg(long, value_name = "INTERFACE", group = "input")]
    microphone_interface: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), u8> {
    let _file = "../songs/White Teeth_Ryan Beatty.wav".to_string();
    let _song: String = "White Teeth".to_string();
    let _artist: String = "Ryan Beatty".to_string();

    assert_eq!(get_song_title_artist(&_file), Ok((_song, _artist)));

    // Parse command line arguments.
    let args = Args::parse();

    // Always add any new songs to the database before trying to identify a snippet.
    // If there is any failures, `?` will percolate up to be main()'s return
    let add_song_files = args.add_song;

    // If there is more than one song, attempt to process them concurrently.
    if add_song_files.len() > 1 {
        // `.await` will pause execution of the `main()` task if `add_song...` is not ready,
        // which is desired since we do not want to try to identify songs until the database
        // is fully set up
        add_song_files_concurrently(&add_song_files).await?

    // Regular, sequential processing of one song.
    } else if add_song_files.len() > 0 {
        let song_file_path: &String = &add_song_files[0];

        let (song_title, song_artist): (String, String) = get_song_title_artist(song_file_path)?;

        let uri =
            spotify_utils::get_track_uri(song_title.to_string(), song_artist.to_string()).await;

        let song_id = db_utils::store_song(&song_title, &song_artist, uri)?;

        let fingerprint =
            fingerprint::fingerprint_audio(song_file_path.to_string(), song_id).await?;

        db_utils::store_fingerprints(fingerprint)?
    }

    // Identify a song based on a .wav file
    if let Some(id_song_file) = args.id_song {
        let song_id = rand::random::<u32>();

        let sample_fingerprint_map: HashMap<u32, KeyAudioPoint> =
            fingerprint::fingerprint_audio(id_song_file.to_string(), song_id).await?;

        let fingerprint: HashMap<u32, u32> = sample_fingerprint_map
            .into_iter()
            .map(|(hash, key_audio_point)| (hash, key_audio_point.anchor_time_ms as u32))
            .collect();

        let matches: Vec<Match> = shazam::find_matches_from_fingerprint(fingerprint)?;

        if matches.len() < 1 {
            eprintln!("No matches found for `{}`!", id_song_file);
            return Err(NO_SONG_MATCH_ERROR);
        }

        #[cfg(debug_assertions)]
        for a_match in &matches {
            println!("{:?}", a_match);
        }

        let best_match: Match = matches[0].clone();
        
        if best_match.score < MATCH_SCORE_THRESHOLD {
            eprintln!("No LIKELY match found for `{}`!", id_song_file);
            println!("Best match was: {:?}", best_match);
            return Err(NO_SONG_MATCH_ERROR);
        }

        if let Some(uri) = best_match.spotify_uri {
            if uri.is_empty() {
                let uri: String =
                    spotify_utils::play_song(&best_match.song_title, &best_match.song_artist)
                        .await?;

                db_utils::update_song_uri(&best_match.song_title, &best_match.song_artist, uri)?;
            } else {
                spotify_utils::play_song_from_uri(&uri).await?;
            }
        } else {
            let uri: String =
                spotify_utils::play_song(&best_match.song_title, &best_match.song_artist).await?;

            db_utils::update_song_uri(&best_match.song_title, &best_match.song_artist, uri)?;
        }
    }

    Ok(())
}

/// Concurrently process a vector of Strings that are paths to .wav files, appropriately fetching
/// Spotify track URIs, fingerprinting the audio, and storing to database.
async fn add_song_files_concurrently(songs_to_add: &Vec<String>) -> Result<(), u8> {
    let mut get_uri_tasks = Vec::with_capacity(songs_to_add.len());
    let mut title_artist_file_vec = Vec::<(String, String, String)>::new();

    for song_file_path in songs_to_add {
        // Parsing the song title and arist is low-work => process sequentially
        let (song_title, song_artist) = match get_song_title_artist(song_file_path) {
            Ok((song_title, song_artist)) => {
                title_artist_file_vec.push((
                    song_title.to_string(),
                    song_artist.to_string(),
                    song_file_path.to_string(),
                ));

                (song_title, song_artist)
            }
            Err(_) => {
                eprintln!(
                    "Could not parse title and/or artist for `{}`! Skipping...",
                    song_file_path
                );
                continue;
            }
        };

        let moveable_song = song_title.to_string();
        let moveable_artist = song_artist.to_string();

        // `get_track_uri` relies on the Spotify API's response time => process concurrently.
        // Calling `tokio::spawn` immediately begins running in background
        get_uri_tasks.push((
            song_title.to_string(),
            tokio::spawn(spotify_utils::get_track_uri(moveable_song, moveable_artist)),
        ))
    }

    // Storing song metadata to happen sequentially to prevent data races.
    // So, join the threads before
    let mut uris_vec = Vec::<Option<String>>::new();
    for (song_title, task) in get_uri_tasks {
        match task.await {
            Ok(option) => uris_vec.push(option),
            Err(_) => {
                eprintln!(
                    "ERROR: Could not join spotify_utils::get_track_uri() task for song \
                `{}`",
                    song_title
                );
            }
        }
    }

    let mut song_ids_file_vec = Vec::<(u32, String)>::new();
    for (uri, (title, artist, file)) in uris_vec.into_iter().zip(&title_artist_file_vec) {
        match db_utils::store_song(&title, &artist, uri) {
            Ok(song_id) => {
                song_ids_file_vec.push((song_id, file.to_string()));
            }
            Err(_) => {}
        }
    }

    // Pause to handle errors here.
    if song_ids_file_vec.is_empty() {
        return Err(DATABASE_INSERT_ERROR);
    }

    // Fingerprinting does not interact with database; safe to be concurrent.
    let mut fingerprinting_tasks = Vec::new();
    for (song_id, song_file_path) in song_ids_file_vec {
        let file_path = song_file_path.to_string();

        fingerprinting_tasks.push((
            song_file_path.to_string(),
            tokio::spawn(fingerprint::fingerprint_audio(file_path, song_id)),
        ))
    }

    for (song_file_path, fingerprint_join_handle) in fingerprinting_tasks {
        match fingerprint_join_handle.await {
            Ok(fingerprint_result) => match fingerprint_result {
                Ok(fingerprint) => db_utils::store_fingerprints(fingerprint).unwrap_or_else(|_| {
                    eprintln!(
                        "ERROR: Could not save fingerprints for `{}` to database!",
                        song_file_path
                    );
                }),
                Err(_) => {
                    eprintln!(
                        "ERROR: Could not generate fingerprint for `{}`",
                        song_file_path
                    );
                }
            },

            Err(_) => {
                eprintln!(
                    "ERROR: Could not join fingerprint_audio() task for `{}`",
                    song_file_path
                );
            }
        }
    }

    Ok(())
}

fn get_song_title_artist(file_path: &String) -> Result<(String, String), u8> {
    match File::open(file_path) {
        Err(_) => {
            eprintln!("ERROR: Cannot open file: `{}`", file_path);
            return Err(FILE_NOT_FOUND);
        }
        Ok(_file) => {}
    }

    let file_as_path = Path::new(file_path);

    let file_name: &str = match file_as_path.file_stem() {
        None => {
            eprintln!("ERROR: `{}` does not have a file name!", file_path);
            return Err(INCOMPATIBLE_FILE_ERROR);
        }

        Some(file_name) => file_name.to_str().unwrap(),
    };

    let vec_names: Vec<&str> = file_name.split('_').collect();

    if vec_names.len() < 2 {
        eprintln!(
            "ERROR: `{}` does not have underscore-delimited parts!",
            file_path
        );
        println!("Example: `title_artist.wav`");
        return Err(INCOMPATIBLE_FILE_ERROR);
    }

    let (song_name, artist_name) = (vec_names[0], vec_names[1]);

    Ok((song_name.to_string(), artist_name.to_string()))
}
