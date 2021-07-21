mod api;

use rocket;

#[rocket::main]
async fn main() {
    api::rocket().launch().await.unwrap();
}
