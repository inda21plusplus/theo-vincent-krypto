#[macro_use]
extern crate rocket;

use rocket::data::FromData;
use rocket::form::Form;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

use rocket_auth::prelude::*;

use types::{CreateInfo, FileData, FileInfo, LoginInfo};

mod data;

use data::Database;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/push", format = "json", data = "<file>")]
fn push(file: Json<FileData>) {
    println!("{}", file.name);
    println!("{}", std::str::from_utf8(&file.contents[..]).unwrap());
}

#[post("/pull", format = "json", data = "<info>")]
fn pull(info: Json<FileInfo>) {
    dbg!(info);
}

#[post("/create", data = "<form>")]
async fn create(form: Form<Signup>, auth: Auth<'_>) -> Result<&'static str, Error> {
    auth.signup(&form).await?;
    auth.login(&form.into());
    Ok("You signed up.")
}

#[post("/login", data = "<form>")]
async fn login(
    form: rocket::serde::json::Json<Login>,
    auth: Auth<'_>,
) -> Result<&'static str, Error> {
    auth.login(&form).await?;
    Ok("You're logged in.")
}

#[tokio::main]
async fn launch() -> Result<(), Error> {
    let users = Users::open_rusqlite("mydb.db").await?;

    rocket::build()
        .mount("/", routes![push, pull, signup, login, logout])
        .manage(users)
        .launch();

    Ok(())
}
