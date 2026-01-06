// file: src/recognizer/spectrogram.rs

use crate::recognizer::declarations::SPECTROGRAM_GENERATION_FAILURE;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::PI;

const MAX_FREQUENCY: f64 = 5000.0; // 5 kHz
const DSP_RATIO: u32 = 4;
const WINDOW_SIZE: usize = 1024;
const SCROLL_SIZE: usize = WINDOW_SIZE / 2; // allow overlap

pub struct Peak {
    pub frequency: f64,
    pub time_sec: f64,
}

/// Given a digitized audio sample, produce the spectrogram as a map of floating point magnitudes.
pub fn gen_spectrogram(sample: Vec<f64>, sample_rate: u32) -> Result<Vec<Vec<f64>>, u8> {
    let mut spectrogram: Vec<Vec<f64>> = Vec::new();

    if sample.len() == 0 {
        return Ok(spectrogram);
    }

    let filtered_sample = lowpass_filter(sample, MAX_FREQUENCY, sample_rate);

    let downsampled: Vec<f64> = downsample(filtered_sample, sample_rate, sample_rate / DSP_RATIO)?;

    let hanning_window: Vec<f64> = (0..WINDOW_SIZE)
        .map(|idx| {
            let theta = 2.0 * PI * idx as f64 / (WINDOW_SIZE - 1) as f64;

            0.5 - 0.5 * f64::cos(theta)
        })
        .collect();

    for start in (0..downsampled.len())
        .take_while(|idx| (idx + WINDOW_SIZE) < downsampled.len())
        .step_by(SCROLL_SIZE)
    {
        let end = start + WINDOW_SIZE;

        // Apply the Hanning window to a section of the downsampled data
        let mut frame: Vec<Complex<f64>> = downsampled[start..end]
            .into_iter()
            .enumerate()
            .map(|(idx, value)| Complex {
                re: value * hanning_window[idx],
                im: 0.0,
            })
            .collect();

        let fft = FftPlanner::new().plan_fft_forward(frame.len());

        fft.process(&mut frame);

        let magnitude: Vec<f64> = frame.into_iter().map(|val| val.norm() as f64).collect();

        spectrogram.push(magnitude);
    }

    Ok(spectrogram)
}

// Remove frequency values below a certain threshold.
fn lowpass_filter(input: Vec<f64>, cutoff_frequency: f64, sample_rate: u32) -> Vec<f64> {
    let time_constant = 1.0 / (2.0 * PI * cutoff_frequency);
    let dt = 1.0 / sample_rate as f64;
    let alpha = dt / (time_constant + dt);

    let mut previous_output: f64 = 0.0;

    input
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let new_value: f64;

            if index == 0 {
                new_value = value * alpha;
            } else {
                new_value = value * alpha + (1.0 - alpha) * previous_output;
            }

            previous_output = new_value;
            new_value
        })
        .collect::<Vec<f64>>()
}

// Reduce the number of samples in an audio input, compressing data to improve performance.
fn downsample(input: Vec<f64>, sample_rate: u32, target_sample_rate: u32) -> Result<Vec<f64>, u8> {
    if target_sample_rate > sample_rate {
        eprintln!("Target sample rate must be less than or equal to original sample rate");
        return Err(SPECTROGRAM_GENERATION_FAILURE);
    }

    // Check that integer division did not result in 0
    let sample_ratio = sample_rate / target_sample_rate;
    if sample_ratio <= 0 {
        eprintln!("Invalid ratio calculated from sample rates");
        return Err(SPECTROGRAM_GENERATION_FAILURE);
    }

    let mut resampled: Vec<f64> = Vec::<f64>::new();
    let input_length = input.len();

    for i in (0..input_length).step_by(sample_ratio as usize) {
        let mut end: usize = i + sample_ratio as usize;

        if end > input_length {
            end = input_length
        }

        let sum: f64 = input[i..end].iter().sum();
        let average: f64 = sum / (end - i) as f64;

        resampled.push(average);
    }

    Ok(resampled)
}

/// Find the "characteristic" components of an audio-source spectrogram by finding the
/// frequencies with the largest magnitude in the set of frequency ranges human ears perceive the
/// best.
pub fn get_peaks(spectrogram: Vec<Vec<f64>>, duration: f64, sample_rate: u32) -> Vec<Peak> {
    let mut peaks = Vec::<Peak>::new();

    if spectrogram.len() < 1 {
        return peaks;
    }

    struct MaxMagnitude {
        magnitude: f64,
        frequency_idx: usize,
    }

    struct FrequencyBand {
        min_frequency: usize,
        max_frequency: usize,
    }

    let frequency_bands = vec![
        FrequencyBand {
            min_frequency: 0,
            max_frequency: 10,
        },
        FrequencyBand {
            min_frequency: 10,
            max_frequency: 20,
        },
        FrequencyBand {
            min_frequency: 20,
            max_frequency: 40,
        },
        FrequencyBand {
            min_frequency: 40,
            max_frequency: 80,
        },
        FrequencyBand {
            min_frequency: 80,
            max_frequency: 160,
        },
        FrequencyBand {
            min_frequency: 160,
            max_frequency: 512,
        },
    ];

    let frame_duration: f64 = duration / (spectrogram.len() as f64);

    let effective_sample_rate = sample_rate as f64 / DSP_RATIO as f64;
    let frequency_resolution = effective_sample_rate / WINDOW_SIZE as f64;

    // Iterate over every frame in the spectrogram. For each frame, find the maximum magnitudes
    // in each frequency band. Then, take the average of all those maximums to serve as a
    // threshold for values to retain.
    for (frame_idx, frame) in spectrogram.into_iter().enumerate() {
        let mut max_magnitudes = Vec::<f64>::new();
        // let mut frequency_indices = Vec::<usize>::new();

        let max_magnitudes_in_frame: Vec<MaxMagnitude> = (&frequency_bands)
            .into_iter()
            .map(|band: &FrequencyBand| {
                let mut max_magnitude: f64 = frame[band.min_frequency];
                let mut max_magnitude_idx: usize = band.min_frequency;

                for (idx, val) in (&frame[band.min_frequency..band.max_frequency])
                    .into_iter()
                    .skip(1)
                    .enumerate()
                {
                    if *val > max_magnitude {
                        max_magnitude = *val;
                        max_magnitude_idx = band.min_frequency + idx;
                    }
                }

                max_magnitudes.push(max_magnitude);

                MaxMagnitude {
                    magnitude: max_magnitude,
                    frequency_idx: max_magnitude_idx,
                }
            })
            .collect();

        let sum: f64 = (&max_magnitudes).iter().sum();
        let average: f64 = sum / max_magnitudes.len() as f64;

        // Only add peaks that exceed this average value
        for max_mag_struct in max_magnitudes_in_frame {
            let (magnitude, freq_idx) = (max_mag_struct.magnitude, max_mag_struct.frequency_idx);

            if magnitude > average {
                let peak_time = frame_duration * frame_idx as f64;
                let peak_frequency = frequency_resolution * freq_idx as f64;

                peaks.push(Peak {
                    frequency: peak_frequency,
                    time_sec: peak_time,
                });
            }
        }
    }

    peaks
}
