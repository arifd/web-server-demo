mod auth;
mod db;

#[cfg(test)]
mod tests;

use once_cell::sync::Lazy;
use rocket::{
    form::{Form, FromForm},
    fs::FileServer,
    get,
    http::{ContentType, Cookie, CookieJar, Status},
    post, request,
    request::FromRequest,
    response::{content::Json, Redirect, Responder},
    routes,
    serde::Serialize,
    Build, Request, Rocket,
};
use serde_json::{json, Value};
use std::fmt::Debug;
use std::net::IpAddr;
use tera::Tera;

//===========================================================================//
// TODO: HTTPS!!                                                             //
// EITHER TLS: https://rocket.rs/v0.5-rc/guide/configuration/#tls            //
// AND/OR REDIRECT HTTP TO HTTPS                                             //
//===========================================================================//

//===========================================================================//
// NOTE: Unwraps are fine for now because Rocket manages panics and returns  //
// appropriate errors to the client                                          //
//===========================================================================//

//===========================================================================//
// NOTE: The architecture of the interaction between client and server is    //
// Still not ideal. I'm not sure I want redirects, and I think i might prefer//
// to do the form submission manually client-side using JS. have this server //
// always return JSON as a principle.                                        //
// (apart from when it delivers the initial web app)                         //
//===========================================================================//

//===========================================================================//
// NOTE: Users want to be identified by id, not by username!                 //
// Which are cross referenced with theusers DB (along with JWT verification).//
// If JWT fails OR db is deleted, user permissions fail.                     //
//===========================================================================//

//===========================================================================//
// TODO: Reconsider database choice                                          //
//===========================================================================//

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
    Lazy::new(|| Tera::new("templates/**/*").expect("Error with templates"));

//===========================================================================//
// LAUNCH                                                                    //
//===========================================================================//

// #[launch]
pub fn rocket() -> Rocket<Build> {
    let address: IpAddr = "0.0.0.0".parse().unwrap();
    let port = std::env::var_os("PORT").unwrap_or_else(|| "8080".into());
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
                // tweets,
                failure,
                delete,
            ],
        )
    // TODO: Consider /user path where all things need to be authenticated with the user API for things like posting tweet and getting tweets
}

//===========================================================================//
// HOME                                                                      //
//===========================================================================//

// TODO: precompile any static HTML in the build.rs

/// Root path without authentication will deliver the web app
#[get("/", rank = 2)]
async fn home() -> (ContentType, String) {
    (
        ContentType::HTML,
        TEMPLATES
            .render("home.html.tera", &tera::Context::new())
            .unwrap(),
    )
}

/// Root path with authentication token will deliver your dashboard data
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

struct User {
    username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(cookie) = req.cookies().get("Authorization") {
            match auth::verify_decode_login_token(cookie.value()) {
                Ok(claims) => {
                    return request::Outcome::Success(User {
                        username: claims.sub,
                    });
                }
                Err(err) => println!("Auth token invalid: {}", err),
            }
        } else {
            println!("No auth token present");
        }

        request::Outcome::Forward(())
    }
}

//===========================================================================//
// ACCOUNT                                                                   //
//===========================================================================//

#[post("/signup", data = "<form>")]
async fn post_signup(form: Form<SignupForm>) -> (Status, Value) {
    // Check username doesn't already exist
    if db::user_exists(&form.username) {
        return (
            Status::BadRequest,
            json!({"error": "This username is already taken. Please choose another"}),
        );
    }

    let pwdhash = auth::hash_password(&form.password);
    db::store_user(&form.username, &pwdhash).unwrap();

    let token = auth::generate_login_token(&form.username).unwrap();
    // cookies.add(Cookie::new("Authorization", token));
    (Status::Accepted, json!({ "jwt": token }))
    // (Status::Accepted, Value::Null)
}

#[post("/login", data = "<form>")]
async fn post_login(form: Form<LoginForm>, _cookies: &CookieJar<'_>) -> (Status, Value) {
    // Check user and password match what is in the DB
    if let Ok(pwdhash) = db::get_password_hash(&form.username) {
        if let Ok(true) = auth::verify_pwdhash(&pwdhash, &form.password) {
            let token = auth::generate_login_token(&form.username).unwrap();
            // cookies.add(Cookie::new("Authorization", token));
            return (Status::Accepted, json!({ "jwt": token }));
            // return (Status::Accepted, Value::Null);
        }
    }

    (
        Status::Forbidden,
        json!({"error": "The username and password you entered did not match our records. Please double-check and try again."}),
    )
}

#[get("/failure")]
async fn failure() -> String {
    String::from("failure")
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
// #[get("/tweets")]
// async fn tweets() -> JsonResponse {
//     let tweets = db::get_tweets().unwrap();
//     let json = serde_json::to_string(&tweets).unwrap();

// }

//===========================================================================//
// MISC                                                                      //
//===========================================================================//

#[get("/delete")] // GET-ing for easy browser interface
async fn delete() -> String {
    match std::fs::remove_file("cwitter.db") {
        Ok(()) => "success".to_string(),
        Err(err) => format!("err: {:?}", err),
    }
}

#[get("/test")]
async fn test(cookies: &CookieJar<'_>) -> Option<String> {
    cookies
        .get("Authorization")
        .map(|c| format!("cookie found: {}", c.value()))
}
