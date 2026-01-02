/*
file: src/recognizer/fingerprint.rs
*/
use std::collections::HashMap;

use crate::recognizer::spectrogram::{gen_spectrogram, get_peaks, Peak};
use crate::recognizer::wav;

const MAX_FREQUENCY_BITS: i32 = 9;
const MAX_TIME_DELTA_BITS: i32 = 14;
const TARGET_ZONE_SIZE: usize = 5;

pub(crate) struct KeyAudioPoint {
    pub(crate) anchor_time_ms: i32,
    pub(crate) song_id: i32,
}

/** Generates the "fingerprint" of an audio file, returning a hash map where
the key is the unique hash generated from anchor-target pairs and the value is a list of anchor
times and the associated song.
*/
pub(crate) fn fingerprint_audio(file_path: &String, song_id: u32) -> Result<HashMap<u32,
    KeyAudioPoint>, u8> {
    let wav_info: wav::WavInfo = wav::get_wav_info(&file_path)?;
    
    let mut fingerprint: HashMap<u32, KeyAudioPoint> = HashMap::new();

    let left_spectrogram =
        gen_spectrogram(wav_info.left_channel_samples, wav_info.spec.sample_rate)?;

    let left_peaks = get_peaks(
        left_spectrogram,
        wav_info.duration_sec,
        wav_info.spec.sample_rate,
    );

    fingerprint.extend(gen_fingerprints(left_peaks, song_id));

    let right_spectrogram =
        gen_spectrogram(wav_info.right_channel_samples, wav_info.spec.sample_rate)?;

    let right_peaks = get_peaks(
        right_spectrogram,
        wav_info.duration_sec,
        wav_info.spec.sample_rate,
    );

    fingerprint.extend(gen_fingerprints(right_peaks, song_id));

    Ok(fingerprint)
}


pub(crate) fn gen_fingerprints(peaks: Vec<Peak>, song_id: u32) -> HashMap<u32, KeyAudioPoint> {
    let mut fingerprints = HashMap::<u32, KeyAudioPoint>::new();

    for (i, anchor) in (&peaks).into_iter().enumerate() {
        for j in ((i + 1)..peaks.len()).take_while(|&j| j <= i + TARGET_ZONE_SIZE) {
            let target: &Peak = &peaks[j];

            let hash = gen_hash(&anchor, &target);
            let anchor_time_ms: i32 = (anchor.time_sec * 1000.) as i32;

            fingerprints.insert(
                hash,
                KeyAudioPoint {
                    anchor_time_ms,
                    song_id : song_id as i32,
                },
            );
        }
    }

    fingerprints
}


fn gen_hash(anchor: &Peak, target: &Peak) -> u32 {
    // Scale down to fit in 9 bits
    let anchor_frequency: u32 = (anchor.frequency / 10.) as u32;
    let target_frequency: u32 = (target.frequency / 10.) as u32;

    let time_delta_ms: u32 = ((target.time_sec - anchor.time_sec) * 1000.) as u32;

    // Mask to fit within bit constraints
    let anchor_frequency_bits = anchor_frequency & ((1 << MAX_FREQUENCY_BITS) - 1); // 9 bits
    let target_frequency_bits = target_frequency & ((1 << MAX_FREQUENCY_BITS) - 1); // 9 bits
    let time_delta_bits = time_delta_ms & ((1 << MAX_TIME_DELTA_BITS) - 1); // 14 bits
                                                                            // (max ~16 seconds)

    // Combine into 32-bit address
    let hash = (anchor_frequency_bits << 23) | (target_frequency_bits << 14) | time_delta_bits;

    hash
}
