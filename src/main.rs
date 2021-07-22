mod api;

#[rocket::main]
async fn main() {
    api::rocket().launch().await.unwrap();
}
