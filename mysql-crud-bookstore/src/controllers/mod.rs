use super::models::BookData;

use dotenv::dotenv;
use std::env;

use mt_logger::*;


use super::schema::Book::dsl::*;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*; //for .execute or other methods

use tide::{Body, Request};

pub async fn create_book(mut req: Request<()>) -> tide::Result<String> {

    mt_new!(None, Level::Trace, OutputStream::Both);
    mt_log!(Level::Trace, "Message {}: a TRACE message", 4);
    mt_flush!().unwrap();

    let book_to_create: BookData = req.body_json().await?;
    let connection = establish_connection();

    let _ = diesel::insert_into(Book)
        .values(&book_to_create)
        .execute(&connection);

    Ok(format!(
        "New book name is: {}",
        book_to_create.name.unwrap()
    ))
}

pub async fn update_book(mut req: Request<()>) -> tide::Result<String> {
    let book_to_update: BookData = req.body_json().await?;
    let connection = establish_connection();

    let target = Book.find(book_to_update.id.unwrap());
    let update_count = diesel::update(target)
        .set(&book_to_update)
        .execute(&connection)?;

    Ok(format!(
        "Updated book name is: {} and update count: {}",
        book_to_update.name.unwrap(),
        update_count
    ))
}

pub async fn get_all_books(req: Request<()>) -> tide::Result<Body> {
    let connection = establish_connection();

    let all_books = Book.load::<BookData>(&connection)?;
    Body::from_json(&all_books)
}

pub async fn get_book_by_id(req: Request<()>) -> tide::Result<Body> {
    let book_id: i32 = req.param("bookId")?.parse()?;

    let connection = establish_connection();

    let found_book: BookData = Book.find(book_id).first(&connection)?;
    Body::from_json(&found_book)
}

pub async fn delete_book_by_id(req: Request<()>) -> tide::Result<String> {
    let book_id: i32 = req.param("bookId")?.parse()?;

    let connection = establish_connection();

    let delete_count = diesel::delete(Book.find(book_id)).execute(&connection)?;

    Ok(format!("Deleted book count is: {}", delete_count))
}

fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
