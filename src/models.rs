use super::schema::pages;
use serde::Deserialize;
use serde::Serialize;
use yarte::Template;

#[derive(Queryable, Deserialize, Serialize, Template)]
#[template(path = "hello")]
pub struct Page {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub link: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "pages"]
pub struct NewPage {
    pub title: String,
    pub description: String,
    pub link: String,
}
