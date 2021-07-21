// TEMPORARY
#[allow(dead_code, unused_variables, unused_imports)]
// END-TEMPORARY
mod auth;
mod db;

use argon2;
use once_cell::sync::Lazy;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{ContentType, Cookie, CookieJar, Header};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket::{Response, *};
use std::fmt::Debug;
use std::io::Cursor;
use std::net::IpAddr;
use tera::Tera;

//===========================================================================//
// NOTES: Unwraps are fine for now because Rocket manages panics and returns //
// appropriate errors to the client                                          //
//===========================================================================//

#[macro_export]
macro_rules! path {
    ($path: literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/", $path,)
    };
}

#[derive(FromForm)]
pub struct PostTweetForm {
    body: String,
}

#[derive(Debug, Serialize)]
pub struct Tweet {
    username: String,
    body: String,
}

#[derive(FromForm)]
pub struct SignupForm {
    username: String,
    password: String,
}

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

static TEMPLATES: Lazy<Tera> =
    Lazy::new(|| Tera::new(path!("templates/**/*")).expect("Error with templates"));

//===========================================================================//
// LAUNCH                                                                    //
//===========================================================================//

// #[launch]
pub fn rocket() -> Rocket<Build> {
    let address: IpAddr = "0.0.0.0".parse().unwrap();
    let port = std::env::var_os("PORT").unwrap_or("8080".into());
    let port: u16 = port
        .to_str()
        .expect("Error in env var: PORT")
        .parse()
        .expect("Couldn't parse u16 from env var: PORT");

    let config = rocket::Config {
        address,
        port,
        ..rocket::Config::default()
    };

    rocket::custom(&config)
        .mount("/assets", FileServer::from("assets"))
        .mount(
            "/",
            routes![
                test,
                home,
                dashboard,
                post_signup,
                post_login,
                post,
                tweets,
                failure,
            ],
        )
    // will be a /user path where all things need to be authenticated with the user API for things like posting tweet and getting tweets
}

//===========================================================================//
// HOME                                                                      //
//===========================================================================//

// TODO: use a build.rs script to precompile the home screen and the web client app,
// so that we don't render and minify static data at run-time on each request

/// root path without authentication will deliver the web app
#[get("/", rank = 2)]
async fn home() -> (ContentType, String) {
    (
        ContentType::HTML,
        TEMPLATES
            .render("home.html.tera", &tera::Context::new())
            .unwrap(),
    )
}

struct User {
    username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(cookie) = req.cookies().get("Authorization") {
            if let Ok(claims) = auth::verify_decode_jwt(cookie.value()) {
                return request::Outcome::Success(User {
                    username: claims.sub,
                });
            } else {
                println!("Auth token invalid");
            }
        } else {
            println!("No auth token present");
        }

        request::Outcome::Forward(())
    }
}

/// root path with authentication token will deliver your dashboard data
#[get("/", rank = 1)]
async fn dashboard(_user: User) -> (ContentType, String) {
    let tweets = db::get_tweets().unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("tweets", &tweets);

    (
        ContentType::HTML,
        TEMPLATES.render("dashboard.html.tera", &ctx).unwrap(),
    )
}

//===========================================================================//
// ACCOUNT                                                                   //
//===========================================================================//

#[post("/signup", data = "<form>")]
async fn post_signup(form: Form<SignupForm>) -> Redirect {
    let mut rng = thread_rng();
    let salt: String = (0..8).map(|_| rng.sample(Alphanumeric) as char).collect();
    let pwdhash = argon2::hash_encoded(
        form.password.as_bytes(),
        salt.as_bytes(),
        &argon2::Config::default(),
    )
    .unwrap();
    db::store_user(&form.username, &pwdhash).unwrap();

    Redirect::to("/")
}

#[post("/login", data = "<form>")]
async fn post_login(form: Form<LoginForm>, cookies: &CookieJar<'_>) -> Redirect {
    // Check user and password match what is in the DB
    if let Ok(pwdhash) = db::get_password_hash(&form.username) {
        if let Ok(true) = argon2::verify_encoded(&pwdhash, form.password.as_bytes()) {
            let token = auth::generate_jwt(&form.username).unwrap();
            cookies.add(Cookie::new("Authorization", token));
            return Redirect::to("/");
        }
    }

    Redirect::to("/failure")
}

#[get("/failure")]
async fn failure() -> String {
    String::from("failure")
}

#[get("/test")]
async fn test(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get("auth")
        .map(|c| format!("cookie found: {}", c.value()))
}

//===========================================================================//
// TWEET                                                                     //
//===========================================================================//
#[post("/post", data = "<form>")]
async fn post(form: Form<PostTweetForm>, user: User) -> Redirect {
    let tweet = form.into_inner();
    if !tweet.body.is_empty() {
        let tweet = Tweet {
            username: user.username,
            body: tweet.body,
        };
        db::store_tweet(&tweet).unwrap();
    }
    Redirect::to("/")
}

/// Return all stored tweets as JSON
#[get("/tweets")]
async fn tweets() -> (ContentType, String) {
    let tweets = db::get_tweets().unwrap();
    let json = serde_json::to_string(&tweets).unwrap();
    (ContentType::JSON, json)
}
