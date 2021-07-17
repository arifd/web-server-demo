use once_cell::sync::Lazy;
use r2d2;
use r2d2::Pool;
use r2d2_sqlite;
use r2d2_sqlite::SqliteConnectionManager;
use tide::prelude::*;
use tide::Request;

#[derive(Debug, Deserialize)]
struct Tweet {
    author: String,
    body: String,
}

static DB_CONNECTION_POOL: Lazy<Pool<SqliteConnectionManager>> = Lazy::new(|| {
    // init db
    let db_manager = SqliteConnectionManager::file("tweets.db"); // will create the file if it doesn't exist
    let db_conn_pool = r2d2::Pool::new(db_manager).expect("Failure opening or creating db");
    db_conn_pool
        .get()
        .expect("faliure getting pool for init reasons")
        .execute(
            "CREATE TABLE IF NOT EXISTS tweets (
                id          INTEGER PRIMARY KEY,
                author      TEXT,
                body        TEXT
            )",
            rusqlite::params![],
        )
        .expect("Faliure creating/opening table");
    db_conn_pool
});

#[async_std::main]
async fn main() -> tide::Result<()> {
    let ip = "0.0.0.0".to_string();
    let port = std::env::var_os("PORT").unwrap_or("8080".into());
    let port = port.to_str().expect("error with ENV VAR: PORT");

    let mut app = tide::new();
    app.at("/").get(hello);
    app.at("/tweet").post(tweet);
    app.at("/tweets").get(tweets);
    app.listen(ip + ":" + &port).await?;

    Ok(())
}

async fn hello(_: Request<()>) -> tide::Result {
    Ok("ha11o\n".into())
}

/// store a tweet
async fn tweet(mut req: Request<()>) -> tide::Result {
    // Parse tweet
    let Tweet { author, body } = req.body_json().await?;

    // Save the tweet to the database
    DB_CONNECTION_POOL.get()?.execute(
        "INSERT INTO tweets (author, body) VALUES (?1, ?2)",
        rusqlite::params![author, body],
    )?;

    // Response
    Ok(format!("tweet by {} logged", author).into())
}

/// Return all stored tweets
async fn tweets(_req: Request<()>) -> tide::Result {
    let conn = DB_CONNECTION_POOL.get()?;
    let mut stmt = conn.prepare("SELECT * FROM tweets")?;
    let tweet_iter = stmt.query_map([], |row| {
        Ok(Tweet {
            author: row.get(1)?,
            body: row.get(2)?,
        })
    })?;
    let tweets: Vec<Tweet> = tweet_iter.filter_map(|t| t.ok()).collect();

    // Response
    Ok(format!("{:?}", tweets).into())
}
