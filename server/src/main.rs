#[macro_use]
extern crate rocket;

use rocket::data::FromData;
use rocket::serde::{json::Json, Deserialize, Serialize};
use types::FileData;

struct User {
    name: String,
    passwd: String,
    files: Files,
}

struct Files {}

#[derive(Serialize, Deserialize)]
struct Upload<'r> {
    name: &'r str,
    contents: &'r [u8],
}

struct OwnedUpload {
    name: String,
    contents: Vec<u8>,
}

impl<'r> Upload<'r> {
    pub fn to_owned(&self) -> OwnedUpload {
        let Upload { name, contents } = self;
        OwnedUpload {
            name: name.to_string(),
            contents: contents.to_vec(),
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/push", format = "json", data = "<file>")]
async fn push_file(file: Json<FileData>) -> std::io::Result<()> {
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, push_file])
}
