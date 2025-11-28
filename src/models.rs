use crate::schema::{assignments, people};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = people)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub group_type: String,
    pub active: bool,
}

#[derive(Insertable)]
#[diesel(table_name = people)]
pub struct NewPerson<'a> {
    pub name: &'a str,
    pub group_type: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
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
