#[macro_use]
extern crate diesel;
extern crate dotenv;

mod controllers;
mod models;
mod routes;
pub mod schema;

#[async_std::main]
async fn main() -> tide::Result<()> {
    const PORT: &str = "8081";
    let mut socket: String =  "127.0.0.1:".to_owned();
    socket.push_str(PORT);
    println!("starting web server on {}", socket);

    let mut app = tide::new();
    routes::setup_routes(&mut app);
    app.listen(socket).await?;
    Ok(())
}
