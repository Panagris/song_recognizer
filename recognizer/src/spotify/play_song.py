import sys
import spotipy
from spotipy import Spotify
from spotipy.oauth2 import SpotifyOAuth

# Replace with your actual credentials and redirect URI
CLIENT_ID = "18b3ecc0df694ca7aa9a0127e07e3531"
CLIENT_SECRET = "e284a47688ce45a6917e93b753c86b50"
REDIRECT_URI = "https://www.google.com/"

# Required scopes for playback control
SCOPE = "user-read-playback-state,user-modify-playback-state"

# Initialize Spotipy with OAuth
SPOTIFY = spotipy.Spotify(auth_manager=SpotifyOAuth(client_id=CLIENT_ID,
                                                    client_secret=CLIENT_SECRET,
                                                    redirect_uri=REDIRECT_URI,
                                                    scope=SCOPE))
def main():
    if len(sys.argv) < 2:
        print("Insufficient arguments!")
        exit(-1)

    uri = sys.argv[1]
    try:
        SPOTIFY.start_playback(uris=[uri])
        print(0)
    except spotipy.exceptions.SpotifyException:
        print(-1)
        # print(f"Error playing song: {e}")
        # print("Ensure you have an active Spotify Premium device and the correct scopes are granted.")
    except Exception:
        print(-2)


if __name__ == '__main__':
    main()
