#[macro_use]
extern crate rocket;

use rocket::data::FromData;
use rocket::serde::{json::Json, Deserialize, Serialize};
use types::{FileData, FileInfo};

struct User {
    name: String,
    passwd: String,
    files: Files,
}

struct Files {
    files: Vec<ServerFile>,
}

enum ServerFile {
    Persistent,
    Ephemeral(FileData),
}

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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, push])
}
