// @generated automatically by Diesel CLI.

diesel::table! {
    fingerprints (hash, anchor_time_ms, song_id) {
        hash -> Integer,
        anchor_time_ms -> Integer,
        song_id -> Integer,
    }
}

diesel::table! {
    songs (id) {
        id -> Integer,
        title -> Text,
        artist -> Text,
        album -> Text,
        spotify_uri -> Nullable<Text>,
        song_key -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(fingerprints, songs,);
