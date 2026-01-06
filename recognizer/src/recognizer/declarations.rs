/* file: src/recognizer/declarations.rs

*/

pub const FILE_NOT_FOUND: u8 = 1;
pub const INCOMPATIBLE_FILE_ERROR: u8 = 2;
pub const NO_SONG_MATCH_ERROR: u8 = 3;
pub const SPECTROGRAM_GENERATION_FAILURE: u8 = 4;
pub const DATABASE_INSERT_ERROR: u8 = 5;
pub const DATABASE_QUERY_ERROR: u8 = 6;
pub const SPOTIFY_ERROR: u8 = 7;

pub const MATCH_SCORE_THRESHOLD: f64 = 15.;