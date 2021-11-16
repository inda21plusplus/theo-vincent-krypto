use std::io;
use std::io::prelude::*;
use std::{fs::File, path::Path};

use reqwest::Url;
use types::FileData;

struct ServerInfo {
    push_url: Url,   // upload a file
    pull_url: Url,   // request a file
    delete_url: Url, // delete a file
    get_url: Url,    // request metadata about smh
}

impl ServerInfo {
    pub async fn push_file(&self, path: &Path) -> Result<(), String> {
        let mut file = File::open(path).map_err(|e| format!("Error opening file, error {}", e))?;

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .map_err(|_| String::from("Error reading file"))?; // TODO ADD ENCRYPTION

        let file_name = match path.file_name().and_then(|x| x.to_str()) {
            Some(x) => x.to_string(),
            _ => return Err(String::from("Error getting file name")),
        };

        match reqwest::Client::new()
            .post(self.push_url.clone())
            .json(&FileData {
                name: file_name,
                contents: buffer,
            })
            .send()
            .await
        {
            Ok(response) => {
                println!("Sent, Statuscode: {}", response.status());

                Ok(())
            }
            Err(error) => {
                if let Some(status) = error.status() {
                    Err(format!("Statuscode {}", status))
                } else {
                    if error.is_timeout() {
                        Err(String::from("Timeout"))
                    } else if error.is_decode() {
                        Err(String::from("Decoding"))
                    } else {
                        Err(String::from("Unknown Error"))
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();

    // TODO ADD REAL URL AND METADATA
    let site = ServerInfo {
        push_url: Url::parse("http://127.0.0.1:8000/push")?,
        pull_url: Url::parse("http://127.0.0.1:8000/pull")?,
        delete_url: Url::parse("http://127.0.0.1:8000/delete")?,
        get_url: Url::parse("http://127.0.0.1:8000/get")?,
    };

    // TODO LOGIN

    loop {
        io::stdin().read_line(&mut buffer)?;

        match buffer.split_once(" ") {
            Some((prefix, data)) => match prefix {
                "pull" => {
                    todo!("pull file from server")
                }
                "push" => {
                    if let Err(msg) = site.push_file(Path::new(data.trim())).await {
                        println!("{}", msg)
                    }
                }
                _ => {
                    println!("Invalid prefix")
                }
            },
            _ => match &buffer[0..] {
                "list" => {
                    todo!("List all files")
                }
                _ => {
                    println!("Invalid input");
                }
            },
        }
    }

    Ok(())
}
