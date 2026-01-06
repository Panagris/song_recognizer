// file: src/recognizer/wav.rs

const STARTING_INDEX: i32 = 0;

use crate::recognizer::declarations::{FILE_NOT_FOUND, INCOMPATIBLE_FILE_ERROR};
use hound::WavSpec;
use std::fs::File;

pub struct WavInfo {
    pub spec: WavSpec,
    pub duration_sec: f64,
    pub left_channel_samples: Vec<f64>,
    pub right_channel_samples: Vec<f64>,
}

/// Returns metadata and L-R channel samples of a .wav file given a String path on success.
pub fn get_wav_info(file_path: &String) -> Result<WavInfo, u8> {
    // Check the file exists and is in the .wav format
    match File::open(&file_path) {
        Ok(mut file) => {
            // If the function errors, then the file is definitely not a .wav
            match hound::read_wave_header(&mut file) {
                Ok(_size) => file,
                Err(_) => {
                    eprintln!("File `{}` was not in .wav format!", file_path);
                    return Err(INCOMPATIBLE_FILE_ERROR);
                }
            }
        }
        Err(_) => {
            return Err(FILE_NOT_FOUND);
        }
    };

    let mut wav_reader: hound::WavReader<_> = match hound::WavReader::open(&file_path) {
        Ok(wav_reader) => wav_reader,
        Err(_) => {
            return Err(INCOMPATIBLE_FILE_ERROR);
        }
    };

    let spec = wav_reader.spec();

    if spec.bits_per_sample != 16 {
        eprintln!("WAV bits_per_sample unsupported. Expected 16-bits");
        return Err(INCOMPATIBLE_FILE_ERROR);
    }

    if spec.channels > 2 {
        eprintln!("WAV channels unsupported. Expected 1 or 2");
        return Err(INCOMPATIBLE_FILE_ERROR);
    }

    let duration_sec: f64 = wav_reader.duration() as f64 / spec.sample_rate as f64;

    let all_samples_iter = wav_reader
        .samples::<i16>()
        .filter_map(Result::ok)
        .into_iter()
        .map(|x: i16| x as f64);

    let left_channel_samples: Vec<f64>;

    // If there is one channel, then let the left channel hold all the samples and right be an
    // empty Vec.
    if spec.channels == 1 {
        left_channel_samples = all_samples_iter.collect();

        return Ok(WavInfo {
            spec,
            duration_sec,
            left_channel_samples,
            right_channel_samples: Vec::new(),
        });
    }

    if spec.channels == 2 {
        let right_channel_samples: Vec<f64>;

        // Channel data is interleaved (e.g., Bytes 1 and 2 are for the left channel, Bytes 3 and 4
        // are for the right channel. So, the first 16 bits (Bytes 1 & 2) will be for the left
        // channel, the next 16 for the right channel).

        // With zero-indexing, if `index` is even, then it belongs to the left channel, right
        // channel if odd.
        let mut index: i32 = STARTING_INDEX;
        (left_channel_samples, right_channel_samples) = all_samples_iter.partition(|_: &f64| {
            let bool_return: bool = index % 2 == 0;
            index += 1;
            bool_return
        });

        return Ok(WavInfo {
            spec,
            duration_sec,
            left_channel_samples,
            right_channel_samples,
        });
    }

    Err(INCOMPATIBLE_FILE_ERROR)
}
