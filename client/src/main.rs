use std::fs::File;
use std::io;
use std::io::prelude::*;

use reqwest::Url;

struct ServerInfo {
    pushUrl: Url,
    pullUrl: Url,
}

impl ServerInfo {
    pub async fn push_file(&self, path: String) -> Result<(), String> {
        if let Ok(mut file) = File::open(path) {
            let mut buffer = String::new();

            file.read_to_string(&mut buffer); // TODO ADD ENCRYPTION

            let response = reqwest::Client::new()
                .post(self.pushUrl.clone())
                .body(buffer) // TODO ADD METADATA ABOUT FILE IN JSON .json
                .send()
                .await;

            Ok(())
        } else {
            Err(String::from("Error opening file"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = String::new();

    // TODO ADD REAL URL AND METADATA
    let site = ServerInfo {
        pushUrl: Url::parse("http://localhost/push")?,
        pullUrl: Url::parse("http://localhost/pull")?,
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
                    if let Err(msg) = site.push_file(data.to_string()).await {
                        println!("{}", msg)
                    }
                }
                _ => {
                    println!("Invalid prefix")
                }
            },
            _ => {
                match &buffer[0..] {
                    "list" => {
                        todo!("List all files")
                    }
                    _ => {
                        println!("Invalid input");
                    }
                }
                
            }
        }
    }

    Ok(())
}
