use super::rocket;
use super::TEMPLATES;
use rocket::http::Status;
use rocket::local::blocking::Client;

/// Test that a new user gets given the web app,
/// while a recurring user (with a token) goes straight to the dashboard
#[test]
fn all_phases_of_login() {
    let client = Client::tracked(rocket()).expect("non valid rocket instance");
    // Attempt: first time connection
    let response = client.get("/").dispatch();
    // Expect home page
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.into_string(),
        Some(
            TEMPLATES
                .render("home.html.tera", &tera::Context::new())
                .unwrap()
        ),
    );

    // TODO: Attempt: incorrect sign up

    // TODO: Attempt: correct sign up

    // TODO: Attempt: incorrect log in

    // TODO: Attepmpt: correct log in
}
