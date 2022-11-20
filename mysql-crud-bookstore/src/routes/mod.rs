use super::controllers;

pub fn setup_routes(app: &mut tide::Server<()>) {
    app.at("/book").get(controllers::get_all_books);
    app.at("/book/:bookId").get(controllers::get_book_by_id);

    app.at("/book").post(controllers::create_book);
    app.at("/book").put(controllers::update_book);
    app.at("/book/:bookId")
        .delete(controllers::delete_book_by_id);
}
