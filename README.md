# Song Recognizer

This is a project written in Rust and developed for personal purposes. As 
the title suggests, the program analyzes an audio snippet to determine the 
title and artist of the song associated with an entry in a database of songs.
After identifying the song, the program plays that song on Spotify. 

Many, many thanks to Chigozirim Igweamaka for doing neigh all the "heavy 
lifting" by publicly providing the logic this project utilizes. 
His repository, written in Golang, can be found on [GitHub](https://github.
com/cgzirim/seek-tune).

## Installation

### Requirements
- Rust (version. 1.86.0 or higher)

**Clone the Repository**
```shell
git clone https://github.com/Panagris/song_recognizer.git
cd song_recognizer
```

**Prepare for the Spotify API**
In the cargo project recognizer/, create a `.env` file. Add the following 
lines:
```text
RSPOTIFY_CLIENT_ID=<your_client_id>
RSPOTIFY_CLIENT_SECRET=<your_client_secret>
RSPOTIFY_REDIRECT_URI=<your_application_redirect_url>
```
If you do not have these credentials, follow [this tutorial](https://developer.spotify.com/documentation/web-api/tutorials/getting-started)
by Spotify to get started.

**Prepare for Diesel**
In the same `.env` file used for the Spotify API, add the following line:
```text
DATABASE_URL=db/songs.db
```

## Compilation & Execution
The program executes entirely from the command line, only opening a webpage 
on the system's default web browser to authenticate the Spotify API.

To compile the program without debug prints:
```shell
cargo build -- release
```
To include such prints, omit `-- release`.

The first compilation may take a while as Cargo downloads and installs all 
the requirements described in the [TOML](./recognizer/Cargo.toml) file. 
Subsequent compilations will not take as long.

### Running the Program
The program can be run using `cargo` or by calling the executable generated 
by the previous compilation step. For the latter:
```text
Usage: recognizer [OPTIONS]

Options:
  -a, --add-song <FILE>
          Audio file(s) [.wav] to add to the database. Repeat flag for each additional file
  -i, --id-song <FILE>
          Audio file [.wav] to compare against songs in the database
      --microphone <MICROPHONE>
          Listen on the device's default microphone
      --microphone-interface <INTERFACE>
          Specify a particular interface capable of audio-capture
  -h, --help
          Print help
```

To have cargo run the application: `cargo run`. Passing program arguments is 
done by separating `cargo`'s flags from the program's flags with `--`:
```shell
cargo run -- --add_song <FILE>
```

## The Database
The database [songs.db](./recognizer/db/songs.db) included in this 
repository already possesses some songs in it:

| Song                  | Artist      |
|-----------------------|-------------|
| Ribbons               | Ryan Beatty |
| Bruises Off The Peach | Ryan Beatty |
| Cinnamon Bread        | Ryan Beatty |
| Andromeda             | Ryan Beatty |
| Bright Red            | Ryan Beatty |
| Hunter                | Ryan Beatty |
| White Teeth           | Ryan Beatty |
| Multiple Endings      | Ryan Beatty |
| Little Faith          | Ryan Beatty |
| ENERGY                | Beyoncé     |
| Haircut               | Ryan Beatty |
| Euro                  | Ryan Beatty |
| Cupid                 | Ryan Beatty |

### Why Did I Do This?
I listen to a lot of vinyl records and would like Spotify to count the vinyl
records toward my listening history (for Spotify Wrapped and AirBuds).
Initial attempts used Shazam's Python API, but this failed for songs with
low popularity since they are not in Shazam's API library (specifically,
Ryan Beatty's album _Calico_). So, I sought to build "Shazam" myself.

Though Igweamaka, credited above, had already developed a working 
application that I could have cloned, I wanted a project I could focus on 
during my Winter Break in university. In the previous semester, I had been 
introduced to Rust and have come to appreciate the language (particularly 
many of its in-built / importable resources). I had no prior experience in 
Golang, so porting Igweamaka's repository to Rust gave me some purely visual 
experience with the language.


[//]: # ()
[//]: # (    Time Domain -> Frequency Domain )

[//]: # (Down sample the audio signal to around 12 kHz, cut out signals outside [20 Hz,)

[//]: # (5 kHz])

[//]: # ()
[//]: # (Hamming window function to taper edges of signal that is segmented )

[//]: # ()
[//]: # (FFT on each signal piece and lay it out in a 2D matrix &#40;? don't know why but )

[//]: # (might want to just make it a 1D array&#41; )

[//]: # ()
[//]: # (    Amplitude-Based Identification)

[//]: # (Find the loudest &#40;highest amplitude&#41; frequencies in each of the frequency bands)

[//]: # ()
[//]: # (Very Low: 0 - 10)

[//]: # (Low: 10 - 20)

[//]: # (Low-Mid: 20 - 40)

[//]: # (Mid: 40 - 80)

[//]: # (Mid-High: 80 - 160)

[//]: # (High: 160 - 511)

[//]: # ()
[//]: # (After, for each time slice, calculate the average of the six values and )

[//]: # (remove any values below the average. &#40;might be useful here to convert the )

[//]: # (data into a struct?&#41;)

[//]: # ()
[//]: # (    Fingerprinting)

[//]: # (For each time slice)

[//]: # ()
[//]: # (For each point in a time slice, treat it as an anchor. Find five nearest )

[//]: # (neighbors within a target zone &#40;TBD&#41;. For each anchor target pair, make a )

[//]: # (struct containing the anchor's frequency, the target's frequency, and target )

[//]: # (time minus anchor time. )

[//]: # ()
[//]: # (Make this struct the hash value &#40;key&#41; &#40;TBD: find a way to compact this data&#41; )

[//]: # (for a map that maps the hash value struct to an array of tuples &#40;anchor point )

[//]: # (time, song name&#41;. SAVE TO DB)

[//]: # ()
[//]: # (    Identification)

[//]: # (From input, convert to wave format. Then, process it to get the hash values )

[//]: # (and the times the hash value occurred. Query the database for all )

[//]: # (fingerprint tuples that match the hash)

[//]: # ()
[//]: # (Organize the results by Song into a table of Hash | Time, organized by hash )

[//]: # (matching the audio clip. For the audio clip and a matching Song:)

[//]: # (    from the first matching hash &#40;top of table&#41; and the subsequent hash &#40;a )

[//]: # (hash that matches&#41;, compute the absolute time difference. Then compare the times )

[//]: # (between the audio clip and the Song from the DB. If less than 100, increase )

[//]: # (the Match score for the Song. Keep pointing at the first table entry and )

[//]: # (increment to the next &#40;matching&#41; hash, performing the same differences.)

[//]: # ()
[//]: # (##)