#[macro_use]
extern crate rocket;

use rocket::data::FromData;
use rocket::form::Form;
use rocket::response;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;

use std::sync::Mutex;

use types::{FileData, FileInfo, FileList, FileListEntry};

mod data;
mod file;
mod merkle_tree;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/push", format = "json", data = "<file>")]
fn push(db: &State<Mutex<data::Files>>, file: Json<FileData>) {
    db.lock().unwrap().add_file(file.into_inner());
}

#[get("/pull", format = "json", data = "<info>")]
fn pull(db: &State<Mutex<data::Files>>, info: Json<FileInfo>) -> Json<Option<FileData>> {
    let file = (*db.lock().unwrap()).get_file(info.into_inner());
    Json(file)
}

#[get("/list")]
fn list(db: &State<Mutex<data::Files>>) -> Json<FileList> {
    let files = db.lock().unwrap();

    let mut list = FileList {
        top_hash: files.top_hash().as_ref().to_vec(),
        list: vec![],
    };

    for (name, file) in files.get_all_files() {
        list.list.push(FileListEntry {
            name: name.to_string(),
            size: file.size(),
        })
    }

    Json(list)
}

#[launch]
fn launch() -> _ {
    let file_db = data::Files::new();

    rocket::build()
        .mount("/", routes![index, push, pull, list])
        .manage(Mutex::new(file_db))
}
