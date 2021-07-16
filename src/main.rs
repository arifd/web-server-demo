use std::ops::Add;

use tide::prelude::*;
use tide::Request;

#[derive(Debug, Deserialize)]
struct Animal {
    name: String,
    legs: u8,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let ip = "127.0.0.1".to_string();
    let port = std::env::var_os("PORT").unwrap_or("8080".into());
    let port = port.to_str().expect("error with ENV VAR: PORT");

    let mut app = tide::new();
    app.at("/orders/shoes").post(order_shoes);
    app.at("/test").get(test);
    app.at("/").get(hello);
    app.listen(ip + ":" + &port).await?;
    Ok(())
}

async fn hello(_: Request<()>) -> tide::Result {
    Ok("hello".into())
}

async fn test(_: Request<()>) -> tide::Result {
    Ok("hello fucken world".into())
}

async fn order_shoes(mut req: Request<()>) -> tide::Result {
    let Animal { name, legs } = req.body_json().await?;
    println!("request from: {}", name);
    Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
}
