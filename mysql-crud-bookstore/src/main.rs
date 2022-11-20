#[macro_use]
extern crate diesel;
extern crate dotenv;

mod controllers;
mod models;
mod routes;
pub mod schema;

#[async_std::main]
async fn main() -> tide::Result<()> {
    println!("starting web server on 9009");

    let mut app = tide::new();

    routes::setup_routes(&mut app);

    app.listen("127.0.0.1:9009").await?;

    Ok(())
}
