// file: src/spotify/spotify_utils.rs
// purpose: search for a spotify track given a name and play that song

use crate::recognizer::declarations::SPOTIFY_ERROR;
use std::path::PathBuf;
use std::sync::Arc;
use rspotify::{AuthCodeSpotify, ClientError, ClientResult, Config, Credentials, OAuth,
               prelude::*,
               scopes,
               model::{Country, Market, SearchType, FullTrack, SearchResult, TrackId},
               DEFAULT_API_BASE_URL, DEFAULT_AUTH_BASE_URL, DEFAULT_CACHE_PATH, DEFAULT_PAGINATION_CHUNKS};
use webbrowser;


async fn search_tracks(spotify: &AuthCodeSpotify, track_query: &str) -> ClientResult<SearchResult> {

    // Obtain a token before submitting a request
    match authorize_client(&spotify).await {
        Ok(()) => {},
        Err(client_error) => {
            eprintln!("{:?}", client_error);
            return Err(client_error);
        },
    }

    let result: ClientResult<_> = spotify
        .search(
            track_query,
            SearchType::Track,
            Some(Market::Country(Country::UnitedStates)),
            None,
            Some(5),
            None,
        ).await;

    result
}

//noinspection RsUnresolvedMethod
pub async fn get_track_uri(track_name: &String, artist: &String) -> Option<String> {

    let spotify: AuthCodeSpotify = get_spotify_client();

    let query = format!("track:{} artist:{}", &track_name, artist);

    let result: ClientResult<_> = search_tracks(&spotify, query.as_str()).await;

    let track_results: SearchResult = match result {
        Ok(track_results) => track_results,
        Err(err) => {
            eprintln!("Search error! {err:?}");
            return None
        }
    };

    let desired_track: FullTrack = match track_results {
        SearchResult::Tracks(page_of_tracks) => {
            let tracks = page_of_tracks.items;
            let mut desired_track: Option<FullTrack> = None;

            for track in tracks {
                if track.name.contains(track_name) {
                    desired_track = Some(track);
                    break;
                }
            }

            if desired_track.is_none() {
                eprintln!("No tracks found for track: `{}`!", track_name);
                // return Err(SPOTIFY_ERROR);
                return None
            }

            desired_track.unwrap()
        }
        _ => {
            println!("Not a track!");
            return None
            // return Err(SPOTIFY_ERROR);
        }
    };

    let track_uri = desired_track.id.unwrap();
    println!(
        "Found Track! Name: {}, URI: {}",
        desired_track.name,
        track_uri.to_string()
    );

    Some(track_uri.to_string())
}

/// Play a track on Spotify and return the URI for that track
pub async fn play_song(track_name: &String, artist: &String) -> Result<String, u8> {

    let track_uri = match get_track_uri(track_name, artist).await {
        Some(track_uri) => track_uri,
        None => return Err(SPOTIFY_ERROR),
    };

    let spotify = get_spotify_client();
    do_play_song(&spotify, track_uri.as_str()).await
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

    let config = Config  {
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

pub async fn play_song_from_uri(track_uri: &String) -> Result<(), u8> {
    let spotify = get_spotify_client();

    // // /Users/chiagozieokoye/RustRoverProjects/song_recognizer/.venv
    // let output = Command::new("../.venv/bin/python3")
    //     .arg("./src/spotify/play_song.py")
    //     .arg(track_uri.to_string())
    //     .output();
    //
    // match output {
    //     Ok(output) => {
    //         println!("Python output: {:?}", output);
    //         Ok(())
    //     }
    //     Err(err) => {
    //         eprintln!("Could not get output of python! {:?}", err);
    //         Err(SPOTIFY_ERROR)
    //     }
    // }
    do_play_song(&spotify, track_uri.as_str()).await
        .map_err(|e: ClientError| {
            eprintln!("{:?}", e);
            SPOTIFY_ERROR
        })
}


/* async fn do_stuff() -> ClientResult<()> {
    // The credentials must be available in the environment. Enable the
    // `env-file` feature in order to read them from an `.env` file.
    let creds = Credentials::from_env().unwrap();

    // Using every possible scope
    let scopes = scopes!(
        "user-read-recently-played",
        "user-read-currently-playing",
        "user-read-playback-state",
        "user-read-playback-position",
        "user-modify-playback-state"
    );
    let oauth = OAuth::from_env(scopes).unwrap();


    // let spotify = AuthCodeSpotify::new(creds, oauth);
    let config = Config  {
        api_base_url: String::from(DEFAULT_API_BASE_URL),
        auth_base_url: String::from(DEFAULT_AUTH_BASE_URL),
        cache_path: PathBuf::from(DEFAULT_CACHE_PATH),
        pagination_chunks: DEFAULT_PAGINATION_CHUNKS,
        token_cached: true,
        token_refreshing: true,
        token_callback_fn: Arc::new(None),
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);

    authorize_client(&spotify).await?
} */


//noinspection RsTypeCheck -> Linter unnecessary flags spotify.parse_response_code(&input)
/// Redirect User to authentication page where they copy the URL and paste into terminal to
/// authenticate the application
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
        Ok(_) => {},
        Err(_) => {
            return Err(ClientError::Cli("Error when trying to read from stdin".to_string()))
        }
    }

    match spotify.parse_response_code(&input).ok_or_else(|| 0) {
        Ok(code) => {
            Ok(code)
        },
        Err(_) => {
            Err(ClientError::Cli("Error when trying to parse the response code".to_string()))
        }
    }
}


async fn do_play_song(spotify: &AuthCodeSpotify, track_uri: &str) -> ClientResult<()> {
    authorize_client(spotify).await?;

    let uris = [PlayableId::Track(TrackId::from_uri(track_uri).unwrap(), )];

    spotify.start_uris_playback(uris.into_iter(), None, None, None).await
}


/// Function to run before doing any task with the Spotify client
//noinspection RsUnresolvedMethod -> Linter unnecessary flags spotify...lock()
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
                            Ok(_) => {},
                            Err(_) => {
                                return Err(ClientError::Cli("Error when trying to retrieve the \
                                token".to_string()));
                            }
                        }
                        // submit_request(&spotify, &code).await?;
                    }
                }
            }
        }
        // Otherwise following the usual procedure to get the token.
        _ => {
            let code: String = get_code_from_user(&spotify, &authorize_url)?;

            if let Ok(_) = spotify.request_token(&code).await {} else {
                return Err(
                    ClientError::Cli("Error when trying to retrieve the token".to_string())
                );
            }
        }
    }

    spotify.write_token_cache().await
}