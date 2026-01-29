// @generated automatically by Diesel CLI.

diesel::table! {
    assignments (id) {
        id -> Int4,
        person_id -> Int4,
        task_name -> Text,
        assigned_at -> Timestamp,
    }
}

diesel::table! {
    people (id) {
        id -> Int4,
        name -> Text,
        group_type -> Text,
        active -> Bool,
    }
}

diesel::table! {
    settings (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        email -> Text,
        role -> Text,
        #[max_length = 255]
        password_hash -> Nullable<Varchar>,
    }
}

diesel::joinable!(assignments -> people (person_id));

diesel::allow_tables_to_appear_in_same_query!(assignments, people, settings, users,);
