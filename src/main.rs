use std::fmt::Debug;

use once_cell::sync::Lazy;
use r2d2;
use r2d2::Pool;
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::response::status::Conflict;
use rocket::response::Responder;
use rocket::*;

#[macro_export]
macro_rules! path {
    ($path: literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/", $path,)
    };
}

#[derive(Debug, FromForm)]
struct Tweet {
    author: String,
    body: String,
}

static DB_CONNECTION_POOL: Lazy<Pool<SqliteConnectionManager>> = Lazy::new(|| {
    // Init db
    let db_manager = SqliteConnectionManager::file("tweets.db"); // will create the file if it doesn't exist
    let db_conn_pool = r2d2::Pool::new(db_manager).expect("Failure opening or creating db");
    db_conn_pool
        .get()
        .expect("Failure getting pool for init reasons")
        .execute(
            "CREATE TABLE IF NOT EXISTS tweets (
                id          INTEGER PRIMARY KEY,
                author      TEXT,
                body        TEXT
            )",
            rusqlite::params![],
        )
        .expect("Failure creating/opening table");
    db_conn_pool
});

#[launch]
fn rocket() -> _ {
    // let ip = "0.0.0.0".to_string();
    // let port = std::env::var_os("PORT").unwrap_or("8080".into());
    // let port = port.to_str().expect("error with ENV VAR: PORT");

    rocket::build().mount("/", routes![post, tweet, tweets])
}

#[get("/post")]
async fn post() -> NamedFile {
    NamedFile::open(path!("templates/post.html"))
        .await
        .expect("I expect this to exist")
}

/// Generic error into even worse generic error!
fn bad_err<E: Debug>(err: E) -> Conflict<String> {
    Conflict(Some(format!("{:?}", err)))
}

/// Store a tweet
#[post("/tweet", data = "<tweet>")]
async fn tweet(tweet: Form<Tweet>) -> Result<String, Conflict<String>> {
    // Save the tweet to the database
    DB_CONNECTION_POOL
        .get()
        .map_err(bad_err)?
        .execute(
            "INSERT INTO tweets (author, body) VALUES (?1, ?2)",
            rusqlite::params![tweet.author, tweet.body],
        )
        .map_err(bad_err)?;

    // Response
    Ok(format!("tweet by {} logged", tweet.author))
}

/// Return all stored tweets
#[get("/tweets")]
async fn tweets() -> Result<String, Conflict<String>> {
    let conn = DB_CONNECTION_POOL.get().map_err(bad_err)?;
    let mut stmt = conn.prepare("SELECT * FROM tweets").map_err(bad_err)?;
    let tweet_iter = stmt
        .query_map([], |row| {
            Ok(Tweet {
                author: row.get(1)?,
                body: row.get(2)?,
            })
        })
        .map_err(bad_err)?;

    let tweets: Vec<Tweet> = tweet_iter.filter_map(|t| t.ok()).collect();

    // Response
    Ok(format!("{:?}", tweets))
}
