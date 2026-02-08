// db/schema.rs - Schéma Diesel minimal pour calendar_events
// NOTE: Le vrai schéma complet est dans src/schema.rs (généré par diesel print-schema)
// Ce fichier existe uniquement car certains modules importent depuis db::schema

diesel::table! {
    calendar_events (id) {
        id -> Integer,
        symbol -> Text,
        event_time -> Timestamp,
        impact -> Text,
        description -> Text,
        actual -> Nullable<Double>,
        forecast -> Nullable<Double>,
        previous -> Nullable<Double>,
        created_at -> Timestamp,
    }
}
