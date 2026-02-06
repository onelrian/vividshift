use crate::schema::{assignments, people};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize)]
#[diesel(table_name = people)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub group_type: String,
    pub active: bool,
}

#[derive(Insertable, serde::Deserialize)]
#[diesel(table_name = people)]
pub struct NewPerson {
    pub name: String,
    pub group_type: String,
    pub active: bool,
}

#[derive(AsChangeset, serde::Deserialize)]
#[diesel(table_name = people)]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub group_type: Option<String>,
    pub active: Option<bool>,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(primary_key(key))]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct UserRole {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    #[serde(skip_serializing)]  // Never send password hash to clients
    pub password_hash: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub password_hash: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub role: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize)]
#[diesel(table_name = assignments)]
#[diesel(belongs_to(Person))]
pub struct Assignment {
    pub id: i32,
    pub person_id: i32,
    pub task_name: String,
    pub assigned_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = assignments)]
pub struct NewAssignment<'a> {
    pub person_id: i32,
    pub task_name: &'a str,
    pub assigned_at: NaiveDateTime,
}
