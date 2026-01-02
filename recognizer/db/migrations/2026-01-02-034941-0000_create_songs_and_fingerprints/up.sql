-- Your SQL goes here
CREATE TABLE IF NOT EXISTS fingerprints (
    hash INTEGER NOT NULL,
    anchor_time_ms INTEGER NOT NULL,
    song_id INTEGER NOT NULL,
    PRIMARY KEY (hash, anchor_time_ms, song_id)
);

CREATE TABLE IF NOT EXISTS songs (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    spotify_uri TEXT NOT NULL,
    song_key TEXT NOT NULL UNIQUE
);