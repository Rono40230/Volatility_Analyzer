// @generated automatically by Diesel CLI.

diesel::table! {
    archives (id) {
        id -> Integer,
        title -> Text,
        archive_type -> Text,
        period_start -> Text,
        period_end -> Text,
        comment -> Nullable<Text>,
        created_at -> Text,
        data_json -> Text,
    }
}

diesel::table! {
    calendar_imports (id) {
        id -> Integer,
        name -> Text,
        filename -> Text,
        event_count -> Integer,
        oldest_event_date -> Nullable<Text>,
        newest_event_date -> Nullable<Text>,
        imported_at -> Text,
        is_active -> Bool,
    }
}

diesel::table! {
    calendar_events (id) {
        id -> Integer,
        symbol -> Text,
        event_time -> Timestamp,
        impact -> Text,
        description -> Text,
        actual -> Nullable<Float>,
        forecast -> Nullable<Float>,
        previous -> Nullable<Float>,
        created_at -> Timestamp,
        calendar_import_id -> Integer,
    }
}

diesel::table! {
    predicted_events (id) {
        id -> Integer,
        event_id -> Integer,
        predicted_probability -> Float,
        confidence_score -> Float,
        model_version -> Text,
        predicted_volatility_increase -> Nullable<Float>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    event_metrics (event_type, pair_symbol) {
        event_type -> Text,
        pair_symbol -> Text,
        avg_duration_minutes -> Float,
        peak_time_minutes -> Float,
        return_to_normal_minutes -> Float,
        win_rate -> Float,
        avg_movement_pips -> Float,
        max_movement_pips -> Float,
        whipsaw_rate -> Float,
        best_entry_minutes_before -> Integer,
        contextual_atr_before -> Float,
        contextual_atr_after -> Float,
        atr_increase_ratio -> Float,
        recommended_sl_multiplier -> Float,
        recommended_tp_multiplier -> Float,
        tradability_score -> Float,
        recommendation -> Text,
        sample_size -> Integer,
        last_calculated -> Text,
    }
}

diesel::table! {
    event_movement_quality (id) {
        id -> Nullable<Integer>,
        symbol -> Text,
        event_type -> Text,
        directional_move_rate -> Float,
        whipsaw_rate -> Float,
        avg_pips_moved -> Float,
        success_rate -> Float,
        quality_score -> Float,
        sample_size -> Integer,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::joinable!(predicted_events -> calendar_events (event_id));
diesel::joinable!(calendar_events -> calendar_imports (calendar_import_id));

diesel::allow_tables_to_appear_in_same_query!(
    archives,
    calendar_events,
    calendar_imports,
    predicted_events,
    event_metrics,
    event_movement_quality,
);
