// file: src/spotify/spotify_utils.rs
// purpose: connect to the Spotify Web API using the rspotify crate for functionality including
// searching for track URIs and playing a track

use crate::recognizer::declarations::SPOTIFY_ERROR;
use rspotify::{
    AuthCodeSpotify, ClientError, ClientResult, Config, Credentials, DEFAULT_API_BASE_URL,
    DEFAULT_AUTH_BASE_URL, DEFAULT_CACHE_PATH, DEFAULT_PAGINATION_CHUNKS, OAuth,
    model::{Country, FullTrack, Market, SearchResult, SearchType, TrackId},
    prelude::*,
    scopes,
};
use std::path::PathBuf;
use std::sync::Arc;
use webbrowser;

/// Returns a SearchResult that may contain the top 5 matching Tracks on Spotify for a given query
async fn search_tracks(spotify: &AuthCodeSpotify, track_query: &str) -> ClientResult<SearchResult> {
    // Obtain a token before submitting a request
    match authorize_client(&spotify).await {
        Ok(()) => {}
        Err(client_error) => {
            eprintln!("{:?}", client_error);
            return Err(client_error);
        }
    }

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

//noinspection RsUnresolvedMethod
/// Returns the unique Spotify URI for the top result of a search for a Track based on a name and
/// artist if the track exists.
pub async fn get_track_uri(track_name: String, artist: String, album: String) -> Option<String> {
    let spotify: AuthCodeSpotify = get_spotify_client();

    let query = format!("track:{} artist:{} album:{}", track_name, artist, album);

    let result: ClientResult<_> = search_tracks(&spotify, query.as_str()).await;

    let track_results: SearchResult = match result {
        Ok(track_results) => track_results,
        Err(err) => {
            eprintln!("ERROR: Could not find URI for song `{}`!", track_name);
            eprintln!("{:?}", err);
            return None;
        }
    };

    let desired_track: FullTrack = match track_results {
        SearchResult::Tracks(page_of_tracks) => {
            let found_tracks = page_of_tracks.items;
            let mut desired_track: Option<FullTrack> = None;

            for full_track in found_tracks {
                // HACK: this might not work if two tracks are different because of their case
                // This was done because the result of "Bruises Off the Peach" was "Bruises [o]ff the
                // Peach"
                let found_name: String = full_track.name.to_lowercase();

                if found_name.contains(&track_name.to_lowercase()) {
                    desired_track = Some(full_track);
                    break;
                }
            }

            if desired_track.is_none() {
                eprintln!("No Spotify tracks found for song: `{}`!", track_name);
                return None;
            }

            desired_track.unwrap()
        }
        _ => return None,
    };

    let track_uri = desired_track.id.unwrap();
    println!(
        "Found Track! Name: {}, URI: {}",
        desired_track.name,
        track_uri.to_string()
    );

    Some(track_uri.to_string())
}

/// Given a name and artist, play a track on Spotify. Returns the Spotify URI for that track or
/// error.
pub async fn play_song(track_name: &String, artist: &String, album: &String) -> Result<String, u8> {
    let track_uri = match get_track_uri(
        track_name.to_string(),
        artist.to_string(),
        album.to_string(),
    )
    .await
    {
        Some(track_uri) => track_uri,
        None => return Err(SPOTIFY_ERROR),
    };

    let spotify = get_spotify_client();
    do_play_song(&spotify, track_uri.as_str())
        .await
        .map(|()| track_uri)
        .map_err(|e: ClientError| {
            eprintln!("{:?}", e);
            SPOTIFY_ERROR
        })
}

/// Returns an initialized but not yet authorized Client to handle Spotify API actions
fn get_spotify_client() -> AuthCodeSpotify {
    // The credentials must be available in the environment. Enable the
    // `env-file` feature in order to read them from an `.env` file.
    let creds = Credentials::from_env().unwrap();

    // Using every possible scope
    let scopes = scopes!(
        "user-read-email",
        "user-read-private",
        "user-top-read",
        "user-read-recently-played",
        "user-follow-read",
        "user-library-read",
        "user-read-currently-playing",
        "user-read-playback-state",
        "user-read-playback-position",
        "playlist-read-collaborative",
        "playlist-read-private",
        "user-follow-modify",
        "user-library-modify",
        "user-modify-playback-state",
        "playlist-modify-public",
        "playlist-modify-private",
        "ugc-image-upload"
    );
    let oauth = OAuth::from_env(scopes).unwrap();

    let config = Config {
        api_base_url: DEFAULT_API_BASE_URL.to_string(),
        auth_base_url: DEFAULT_AUTH_BASE_URL.to_string(),
        cache_path: PathBuf::from(DEFAULT_CACHE_PATH),
        pagination_chunks: DEFAULT_PAGINATION_CHUNKS,
        token_cached: true,
        token_refreshing: true,
        token_callback_fn: Arc::new(None),
    };

    AuthCodeSpotify::with_config(creds, oauth, config)
}

/// Plays a song on Spotify with a User's active device given a Spotify URI.
pub async fn play_song_from_uri(track_uri: &String) -> Result<(), u8> {
    let spotify = get_spotify_client();

    do_play_song(&spotify, track_uri.as_str())
        .await
        .map_err(|e: ClientError| {
            eprintln!("{:?}", e);
            SPOTIFY_ERROR
        })
}

/// Performs request for play_song()
async fn do_play_song(spotify: &AuthCodeSpotify, track_uri: &str) -> ClientResult<()> {
    authorize_client(spotify).await?;

    // Before trying to play the song, ensure that there is an active device
    match spotify.device().await {
        Ok(devices) => {
            if devices.is_empty() {
                return Err(ClientError::Cli(
                    "ERROR: User does not have an active Spotify device!".to_string(),
                ));
                // TODO: use Spotify Web SDK to play a random song
            }
        }
        Err(e) => {
            eprintln!("ERROR: Could not get User's active devices!");
            return Err(e);
        }
    }

    let uris = [PlayableId::Track(TrackId::from_uri(track_uri).unwrap())];

    spotify
        .start_uris_playback(uris.into_iter(), None, None, None)
        .await
}

//noinspection RsTypeCheck -> Linter incorrectly flags `spotify.parse_response_code(&input)`
/// Redirect User to authentication page where they copy the URL and paste into terminal to
/// authenticate the application.
fn get_code_from_user(spotify: &AuthCodeSpotify, authorize_url: &str) -> ClientResult<String> {
    match webbrowser::open(&authorize_url) {
        Ok(_) => println!("Opened {} in your browser.", authorize_url),
        Err(why) => eprintln!(
            "Error when trying to open an URL in your browser: {:?}. \
                 Please navigate here manually: {}",
            why, authorize_url
        ),
    }

    println!("Please enter the URL you were redirected to: ");
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(_) => {
            return Err(ClientError::Cli(
                "Error when trying to read from stdin".to_string(),
            ));
        }
    }

    match spotify.parse_response_code(&input).ok_or_else(|| 0) {
        Ok(code) => Ok(code),
        Err(_) => Err(ClientError::Cli(
            "Error when trying to parse the response code".to_string(),
        )),
    }
}

//noinspection RsUnresolvedMethod -> Linter unnecessary flags spotify...lock()
/// Authorize the Spotify client. Run before doing any task with the client.
async fn authorize_client(spotify: &AuthCodeSpotify) -> ClientResult<()> {
    let authorize_url = spotify.get_authorize_url(false)?;

    match spotify.read_token_cache(true).await {
        Ok(Some(new_token)) => {
            let expired = new_token.is_expired();

            // Load token into client regardless of whether it's expired o
            // not, since it will be refreshed later anyway.
            *spotify.get_token().lock().await.unwrap() = Some(new_token);

            if expired {
                // Ensure that we actually got a token from the refetch
                match spotify.refetch_token().await? {
                    Some(refreshed_token) => {
                        *spotify.get_token().lock().await.unwrap() = Some(refreshed_token);
                    }
                    // If not, prompt the user for it
                    None => {
                        println!("Unable to refresh expired token from token cache");
                        println!("Trying normal way!");
                        let code: String = get_code_from_user(&spotify, &authorize_url)?;

                        match spotify.request_token(&code).await {
                            Ok(_) => {}
                            Err(_) => {
                                return Err(ClientError::Cli(
                                    "Error when trying to retrieve the token".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        // Otherwise follow the usual procedure to get the token.
        _ => {
            let code: String = get_code_from_user(&spotify, &authorize_url)?;

            if let Ok(_) = spotify.request_token(&code).await {
            } else {
                return Err(ClientError::Cli(
                    "Error when trying to retrieve the token".to_string(),
                ));
            }
        }
    }

    spotify.write_token_cache().await
}
