// file: src/recognizer/wav.rs

use crate::recognizer::declarations::{FILE_NOT_FOUND, INCOMPATIBLE_FILE_ERROR};
use hound::WavSpec;
use std::fs::File;

pub(crate) struct WavInfo {
    pub spec: WavSpec,
    pub duration_sec: f64,
    pub left_channel_samples: Vec<f64>,
    pub right_channel_samples: Vec<f64>,
}

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
        .into_iter();

    let left_channel_samples: Vec<f64>;

    if spec.channels == 1 {
        left_channel_samples = all_samples_iter.map(|x| x as f64).collect();

        return Ok(WavInfo {
            spec,
            duration_sec,
            left_channel_samples,
            right_channel_samples: Vec::new(),
        });
    }

    // there are 2 channels
    if spec.channels == 2 {
        let right_channel_samples: Vec<f64>;

        // Channel data is interleaved (e.g., Bytes 1 and 2 are for the left channel, Bytes 3 and 4
        // are for the right channel. So, the first 16 bits (Bytes 1 & 2) will be for the left
        // channel, the next 16 for the right channel).
        // let left = wav_reader.samples::<i16>();
        let mut index = -1;
        let (left_channel_i16, right_channel_i16): (Vec<i16>, Vec<i16>) = all_samples_iter
            .partition(|_: &i16| {
                index += 1;
                index % 2 == 0
            });

        left_channel_samples = left_channel_i16
            .into_iter()
            .map(|sample: i16| sample as f64)
            .collect();
        right_channel_samples = right_channel_i16
            .into_iter()
            .map(|sample: i16| sample as f64)
            .collect();

        return Ok(WavInfo {
            spec,
            duration_sec,
            left_channel_samples,
            right_channel_samples,
        });
    }

    Err(INCOMPATIBLE_FILE_ERROR)
}
