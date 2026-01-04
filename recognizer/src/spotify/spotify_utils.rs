// file: src/spotify/spotify_utils.rs
// purpose: search for a spotify track given a name and play that song

use crate::recognizer::declarations::SPOTIFY_ERROR;
use rspotify::model::{FullTrack, SearchResult};
use rspotify::{
    model::{Country, Market, SearchType},
    prelude::*,
    ClientCredsSpotify, ClientResult, Credentials,
};
use std::process::Command;


async fn get_track(spotify: &ClientCredsSpotify, track_query: &str) -> ClientResult<SearchResult> {
    // Obtain a token before submitting a request
    spotify
        .request_token()
        .await
        .expect("Could not obtain token!");

    let result: ClientResult<_> = spotify
        .search(
            track_query,
            SearchType::Track,
            Some(Market::Country(Country::UnitedStates)),
            None,
            Some(5),
            None,
        )
        .await;

    result
}

async fn get_track_uri(track_name: String, artist: String) -> Result<String, u8> {

    let creds = match Credentials::from_env() {
        Some(creds) => creds,
        None => {
            eprintln!("No credentials found! Make sure a `.env` file is configured correctly!");
            return Err(SPOTIFY_ERROR);
        }
    };
    let spotify = ClientCredsSpotify::new(creds);

    let query = format!("track:{} artist:{}", &track_name, artist);

    let result: ClientResult<_> = get_track(&spotify, query.as_str()).await;
    let track_results: SearchResult = match result {
        Ok(track_results) => track_results,
        Err(err) => {
            eprintln!("Search error! {err:?}");
            return Err(SPOTIFY_ERROR);
        }
    };

    let desired_track: FullTrack = match track_results {
        SearchResult::Tracks(page_of_tracks) => {
            let tracks = page_of_tracks.items;
            let mut desired_track: Option<FullTrack> = None;

            for track in tracks {
                if track.name.contains(&track_name) {
                    desired_track = Some(track);
                    break;
                }
            }

            if desired_track.is_none() {
                eprintln!("No tracks found for track: `{}`!", track_name);
                return Err(SPOTIFY_ERROR);
            }

            desired_track.unwrap()
        }
        _ => {
            println!("Not a track!");
            return Err(SPOTIFY_ERROR);
        }
    };

    let track_uri = desired_track.id.unwrap();
    println!(
        "Found Track! Name: {}, URI: {}",
        desired_track.name,
        track_uri.to_string()
    );

    Ok(track_uri.to_string())
}


pub async fn play_song(track_name: String, artist: String) -> Result<(), u8> {

    let track_uri = match get_track_uri(track_name, artist).await {
        Ok(track_uri) => track_uri,
        Err(err) => return Err(err),
    };

    // /Users/chiagozieokoye/RustRoverProjects/song_recognizer/.venv
    let output = Command::new("../.venv/bin/python3")
        .arg("./src/spotify/play_song.py")
        .arg(track_uri.to_string())
        .output();

    match output {
        Ok(output) => {
            println!("Python output: {:?}", output);
            Ok(())
        }
        Err(err) => {
            eprintln!("Could not get output of python! {:?}", err);
            Err(SPOTIFY_ERROR)
        }
    }
}
