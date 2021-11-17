#[macro_use]
extern crate rocket;

use rocket::data::FromData;
use rocket::form::Form;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

use std::sync::Mutex;

use types::{CreateInfo, FileData, FileInfo, LoginInfo};

mod data;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/push", format = "json", data = "<file>")]
fn push(db: &State<Mutex<data::Files>>, file: Json<FileData>) {
    db.lock().unwrap().add_file(file.into_inner());
}

#[post("/pull", format = "json", data = "<info>")]
fn pull(info: Json<FileInfo>) {
    dbg!(info);
}

#[launch]
fn launch() -> _ {
    let file_db = data::Files::default();

    rocket::build()
        .mount("/", routes![push, pull])
        .manage(Mutex::new(file_db))
}
