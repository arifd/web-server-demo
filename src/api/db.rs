use anyhow::Result;
use rusqlite::{params, Connection};
// use once_cell::sync::Lazy;
// use r2d2::Pool;
// use r2d2_sqlite::{rusqlite, SqliteConnectionManager};

use crate::api::Tweet;

//===========================================================================//
// ACCOUNT                                                                   //
//===========================================================================//
pub fn store_user(username: &str, pwdhash: &str) -> Result<()> {
    users_db_connection()?.execute(
        "INSERT INTO users (username, pwdhash) VALUES (?1, ?2)",
        params![username, pwdhash],
    )?;
    Ok(())
}

pub fn get_password_hash(username: &str) -> Result<String> {
    let conn = users_db_connection()?;
    Ok(conn.query_row(
        "SELECT pwdhash FROM users WHERE username=?1",
        params![username],
        |row| row.get(0),
    )?)
}

// /// TODO: NEED SOMEONE TO REVIEW THIS FUNCTION!!
// /// Given a user { username: String, password: String }
// /// Check if the password in the DB for that username matches the one given
// pub fn check_password(user: &User) -> Result<bool> {
//     let conn = users_db_connection()?;

//     let result: Result<String> = conn
//         .query_row(
//             "SELECT password FROM users WHERE username=?1",
//             params![user.username],
//             |row| row.get(0),
//         )
//         .map_err(|_| anyhow::Error::msg("username not found in db"));

//     match result {
//         Err(_) => Ok(false),
//         Ok(db_password) => {
//             if db_password == user.password {
//                 Ok(true)
//             } else {
//                 Ok(false)
//             }
//         }
//     }
// }

//===========================================================================//
// TWEET                                                                     //
//===========================================================================//

/// Save the tweet to the database
pub fn store_tweet(tweet: &Tweet) -> Result<()> {
    tweets_db_connection()?.execute(
        "INSERT INTO tweets (username, body, timestamp) VALUES (?1, ?2, datetime('now'))",
        params![tweet.username, tweet.body],
    )?;
    Ok(())
}

/// Retrieve all tweets in the database
pub fn get_tweets() -> Result<Vec<Tweet>> {
    let conn = tweets_db_connection()?;
    let mut stmt = conn.prepare("SELECT * FROM tweets ORDER BY timestamp DESC")?;
    let tweet_iter = stmt.query_map([], |row| {
        Ok(Tweet {
            username: row.get(1)?,
            body: row.get(2)?,
        })
    })?;

    Ok(tweet_iter.filter_map(|t| t.ok()).collect())
}

//===========================================================================//
// DB MANAGEMENT                                                             //
//===========================================================================//
fn open_db() -> Result<Connection> {
    Ok(Connection::open("cwitter.db")?) // will create if doesn't exist
}

fn tweets_db_connection() -> Result<Connection> {
    let conn = open_db()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tweets (
                id              INTEGER PRIMARY KEY,
                username        TEXT,
                body            TEXT,
                timestamp       INTEGER
            )",
        rusqlite::params![],
    )?;
    Ok(conn)
}

fn users_db_connection() -> Result<Connection> {
    let conn = open_db()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
                        id         INTEGER PRIMARY KEY NOT NULL,
                        username        TEXT,
                        pwdhash         TEXT,
                    )",
        rusqlite::params![],
    )?;
    Ok(conn)
}

///////////////////////////////////////////////////////////////////////////////

// NOTE: I TRIED TO RUN BEFORE I COULD WALK.
// GOING TO PREFER TO OPEN A SINGLE CONNECTION AT EVERY REQUEST

// static DB_CONNECTION_POOL: Lazy<Pool<SqliteConnectionManager>> = Lazy::new(|| {
//     // Init db
//     let db_manager = SqliteConnectionManager::file("tweets.db"); // will create the file if it doesn't exist
//     let db_conn_pool = Pool::new(db_manager).expect("Failure opening or creating db");

//     let db_init = db_conn_pool
//         .get()
//         .expect("Failure getting pool for init reasons");

//     // Credentials Storage
//     db_init
//         .execute(
//             "CREATE TABLE IF NOT EXISTS users (
//                 id         INTEGER PRIMARY KEY NOT NULL,
//                 username        TEXT,
//                 password        TEXT
//             )",
//             rusqlite::params![],
//         )
//         .expect("Failure creating/opening table");

//     // Tweet Storage
//     db_init
//         // .execute(
//         //     "CREATE TABLE IF NOT EXISTS tweets (
//         //         id              INTEGER PRIMARY KEY,
//         //         user_id         INTEGER,
//         //         tweet           TEXT,
//         //         FOREIGN KEY     (user_id) REFERENCES users(id)
//         //     )",
//         //     rusqlite::params![],
//         // )
//         .execute(
//             "CREATE TABLE IF NOT EXISTS tweets (
//                     id              INTEGER PRIMARY KEY,
//                     username        TEXT,
//                     tweet           TEXT,
//                 )",
//             rusqlite::params![],
//         )
//         .expect("Failure creating/opening table");

//     db_conn_pool
// });
