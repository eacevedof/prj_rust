use super::schema::Book;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[table_name = "Book"]
#[primary_key("ID")]
pub struct BookData {
    #[column_name = "ID"]
    #[diesel(deserialize_as = "i32")]
    pub id: Option<i32>,
    #[column_name = "Name"]
    pub name: Option<String>,
    #[column_name = "Author"]
    pub author: Option<String>,
    #[column_name = "Publication"]
    pub publication: Option<String>,
}
