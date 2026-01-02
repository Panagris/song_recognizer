/*
file: src/recognizer/shazam.rs
*/
use std::collections::HashMap;
use crate::db::db_utils;
use crate::recognizer::fingerprint::{KeyAudioPoint};


#[derive(Debug, Clone)]
pub struct Match {
    #[allow(unused)] // TODO use later!
    pub song_title: String,
    #[allow(unused)]
    pub song_artist: String,
    score: f64,
}

/*
pub(crate) fn find_match (sample: Vec<f64>, sample_duration: f64, sample_rate: u32) -> Result<Match, u8> {

    let spectrogram: Vec<Vec<f64>> = gen_spectrogram(sample, sample_rate)?;

    let peaks = get_peaks(spectrogram, sample_duration, sample_rate);

    let song_id = random::<i32>();

    let audio_point_fingerprint = gen_fingerprints(peaks, song_id as u32);

    let fingerprint: HashMap<u32, u32> =
        audio_point_fingerprint.into_iter().map( |(hash, key_audio_point)| {
            (hash, key_audio_point.anchor_time_ms as u32)
    }).collect();

    let matches: Vec<Match> = find_matches_from_fingerprint(fingerprint)?;

    Ok(matches[0].clone())
}*/


pub fn find_matches_from_fingerprint(fingerprint: HashMap<u32, u32>) -> Result<Vec<Match>, u8> {

    let hashes: Vec<i32> = fingerprint.keys().cloned().map(|x: u32| x as i32).collect();

    let matched_fingerprints: HashMap<u32, Vec<KeyAudioPoint>> = db_utils::get_key_audio_points(hashes)?;

    // A HashMap mapping unsigned integers to fixed-sized arrays of size 2.
    // songID -> [(sampleTime, dbTime)
    let mut matches = HashMap::<u32, Vec<[u32; 2]>>::new();

    // songID -> the earliest anchor timestamp
    let mut timestamps = HashMap::<u32, u32>::new();

    // A histogram, songID -> timestamp -> count
    let mut target_zones = HashMap::<u32, HashMap<u32, i32>>::new();


    for (hash, key_audio_point_vec) in matched_fingerprints {

        for key_audio_point in key_audio_point_vec {

            let song_id: u32 = key_audio_point.song_id as u32;
            let anchor_time_ms: u32 = key_audio_point.anchor_time_ms as u32;

            matches.entry(song_id).and_modify(|x: &mut Vec<[u32; 2]>| {

                x.push([fingerprint[&hash], anchor_time_ms])
            }).or_insert(Vec::new());

            // If there is already a timestamp for this hash, see if the new timestamp we
            // encountered is closer. If not, insert the current anchor time
            timestamps.entry(song_id).and_modify(|timestamp: &mut u32| {
                if anchor_time_ms < *timestamp {
                    *timestamp = anchor_time_ms
                }
            }).or_insert(anchor_time_ms);

            // For a song ID, get a Map that represents the count of "matches" for a specific
            // anchor time, either incrementing that count or initializing it to 0.
            target_zones.entry(song_id).and_modify(|zone_map: &mut HashMap<u32, i32>| {

                zone_map.entry(anchor_time_ms).and_modify(|count: &mut i32| {
                    *count += 1;
                }).or_insert(0);
            });

        }
    }

    let scores: HashMap<u32, f64> = analyze_relative_timing(matches);

    let mut match_list = Vec::<Match>::new();

    for (song_id, score) in scores {
        match db_utils::get_song_by_id(song_id) {
            Ok(song) => {
                match_list.push(
                    Match {
                            song_title: song.title,
                            song_artist: song.artist,
                            score
                    }
                );
            },
            Err(error) => {
                eprintln!("Failed to get song: {}", song_id);
                return Err(error);
            }
        }
    }

    match_list.as_mut_slice().sort_by(|a: &Match, b: &Match| {
        b.score.total_cmp(&a.score)
    });


    Ok(match_list)
}

fn analyze_relative_timing(matches: HashMap<u32, Vec<[u32; 2]>>) -> HashMap<u32, f64> {

    let mut scores = HashMap::<u32, f64>::new();

    for (song_id, vector_of_times) in matches {
        let mut offset_counts = HashMap::<i32, i32>::new();

        for time_array in vector_of_times {
            // The time when the amplitudes that generated this unique hash occurred for the
            // sample and when it occurred for the matching value found in the database
            let (sample_time, db_time) = (time_array[0] as i32, time_array[1] as i32);

            let offset: i32 = db_time - sample_time;

            // Bin offsets in 100ms buckets to allow for small timing variations
            let offset_key = offset / 100;
            offset_counts.entry(offset_key).and_modify(|count: &mut i32| {
                *count += 1;
            }).or_insert(0);
        }

        if let Some(key_value_pair) = offset_counts.into_iter()
            .max_by_key(|(_, count)| *count)
        {
            let max_value = key_value_pair.1 as f64;
            scores.entry(song_id).and_modify(|x| {*x = max_value}).or_insert(max_value);
        }
    }

    scores
}
