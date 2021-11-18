use std::io;
use std::io::prelude::*;
use std::process::exit;
use std::{fs::File, path::Path};

use reqwest::Url;
use types::{CreateInfo, FileData, FileInfo, LoginInfo};

use crate::crypto::{decrypt_bytes, hash_password};

mod crypto;

struct ServerInfo {
    login_url: Url,          // login
    create_account_url: Url, // create user account
    push_url: Url,           // upload a file
    pull_url: Url,           // request a file
    delete_url: Url,         // delete a file
    get_url: Url,            // request metadata about smh
    list_url: Url,           // request metadata about smh
}

enum CreateStatus {
    Success,
    AccountTaken,
    Error,
}

enum LoginStatus {
    Success,
    WrongPassword,
    AccountNotFound,
    Error,
}

impl ServerInfo {
    fn get_error_text(error: reqwest::Error) -> String {
        if let Some(status) = error.status() {
            format!("Statuscode {}", status)
        } else {
            if error.is_timeout() {
                String::from("Timeout")
            } else if error.is_decode() {
                String::from("Decoding")
            } else {
                String::from("Unknown Error")
            }
        }
    }

    // 200 = success
    // 401 or 418 = account already exists
    pub async fn create(&self, name: String, password: String) -> Result<CreateStatus, String> {
        match reqwest::Client::new()
            .post(self.create_account_url.clone())
            .json(&CreateInfo {
                name: name.clone(),
                password: crypto::hash_password(name, password),
            })
            .send()
            .await
        {
            Ok(response) => {
                println!("Got, Statuscode: {}", response.status());
                Ok(match response.status().as_u16() {
                    200 => CreateStatus::Success,
                    401 | 418 => CreateStatus::AccountTaken,
                    _ => CreateStatus::Error,
                })
            }
            Err(error) => Err(ServerInfo::get_error_text(error)),
        }
    }

    // 200 = success
    // 403 = wrong password
    // 404 = account not found
    pub async fn login(&self, name: String, password: String) -> Result<LoginStatus, String> {
        match reqwest::Client::new()
            .post(self.login_url.clone())
            .json(&LoginInfo {
                name: name.clone(),
                password: crypto::hash_password(name, password),
            })
            .send()
            .await
        {
            Ok(response) => {
                println!("Got, Statuscode: {}", response.status());
                Ok(match response.status().as_u16() {
                    200 => LoginStatus::Success,
                    403 => LoginStatus::WrongPassword,
                    404 => LoginStatus::AccountNotFound,
                    _ => LoginStatus::Error,
                })
            }
            Err(error) => Err(ServerInfo::get_error_text(error)),
        }
    }

    pub async fn pull_file(
        &self,
        file_name: String,
        //username: String,
        password: String,
        key_pair: &ring::signature::RsaKeyPair,
    ) -> Result<(), String> {
        let hash_name = hash_password(password.clone(), file_name.clone());
        println!("File name {}", hash_name);

        match reqwest::Client::new()
            .get(self.pull_url.clone())
            .json(&FileInfo {
                name_hash: hash_name,
            })
            .send()
            .await
        {
            Ok(response) => {
                println!("Got, Statuscode: {}", response.status());

                if let Some((file_data, tree)) = response
                    .json::<Option<(FileData, types::MerkleData)>>()
                    .await
                    .map_err(|e| format!("Error reading json {}", e))?
                {
                    let mut hash = ring::digest::digest(&ring::digest::SHA256, &file_data.contents);

                    for (side, tree_hash) in tree.hashes {
                        let rhs = hash.as_ref();
                        let lhs = tree_hash;

                        let mut concat = if side == types::Side::Left {
                            lhs.to_vec()
                        } else {
                            rhs.to_vec()
                        };

                        concat.extend_from_slice(if side == types::Side::Left {
                            &rhs
                        } else {
                            &lhs
                        });

                        hash = ring::digest::digest(&ring::digest::SHA256, &concat[..]);
                        println!("{:?}", hash.as_ref())
                    }
                    println!(">> {:?}",  tree.top_hash);

                    if hash.as_ref() != tree.top_hash {
                        return Err(String::from("Invalid hash"));
                    }

                    let decrypted_bytes =
                        decrypt_bytes(file_data.contents, password, file_data.nonce)?;

                    let signature =
                        crypto::sign_file(&decrypted_bytes, file_name.as_bytes(), key_pair)
                            .map_err(|_e| String::from("Error verifying signature"))?;

                    if signature != file_data.signature {
                        return Err(String::from("Invalid signature"));
                    }

                    let mut file = File::create(file_name)
                        .map_err(|e| format!("Error creating file, {}", e))?;

                    file.write_all(&decrypted_bytes)
                        .map_err(|e| format!("Error writing file, {}", e))?;

                    Ok(())
                } else {
                    Err(String::from("No data"))
                }
            }
            Err(error) => Err(ServerInfo::get_error_text(error)),
        }
    }

    pub async fn list_files(&self, password: String) -> Result<(), String> {
        match reqwest::Client::new()
            .get(self.list_url.clone())
            .send()
            .await
        {
            Ok(response) => {
                let resp = response
                    .json::<types::FileList>()
                    .await
                    .map_err(|e| format!("Error parsing response {}", e))?;
                for file in resp.list {
                    if let Ok(file_name_byte_array) =
                        decrypt_bytes(file.name, password.clone(), file.name_nonce)
                    {
                        if let Ok(file_name) = std::str::from_utf8(&file_name_byte_array) {
                            println!("{}", file_name);
                        }
                    }
                }
                Ok(())
            }
            Err(error) => Err(ServerInfo::get_error_text(error)),
        }
    }

    pub async fn push_file(
        &self,
        path: &Path,
        //username: String,
        password: String,
        key_pair: &ring::signature::RsaKeyPair,
    ) -> Result<(), String> {
        let mut file = File::open(path).map_err(|e| format!("Error opening file, {}", e))?;

        let file_name = match path.file_name().and_then(|x| x.to_str()) {
            Some(x) => x.to_string(),
            _ => return Err(String::from("Error getting file name")),
        };

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .map_err(|_| String::from("Error reading file"))?; // TODO ADD ENCRYPTION

        let signature = crypto::sign_file(&buffer, file_name.as_bytes(), key_pair)
            .map_err(|_e| String::from("Error signing file"))?;

        let (nonce, encrypted_file) = crypto::encrypt_bytes(buffer, password.clone())?;
        let (nonce_name, encrypted_file_name) =
            crypto::encrypt_bytes(file_name.as_bytes().to_vec(), password.clone())?;

        let hash_name = hash_password(password.clone(), file_name);
        println!("File name {}", hash_name);

        match reqwest::Client::new()
            .post(self.push_url.clone())
            .json(&FileData {
                name: encrypted_file_name,
                contents: encrypted_file,
                nonce,
                name_nonce: nonce_name,
                name_hash: hash_name,
                signature,
            })
            .send()
            .await
        {
            Ok(response) => {
                println!("Sent, Statuscode: {}", response.status());

                Ok(())
            }
            Err(error) => Err(ServerInfo::get_error_text(error)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO ADD REAL URL
    let main_url = "http://127.0.0.1:8000";
    let site = ServerInfo {
        push_url: Url::parse(&format!("{}/push", main_url))?,
        pull_url: Url::parse(&format!("{}/pull", main_url))?,
        delete_url: Url::parse(&format!("{}/delete", main_url))?,
        get_url: Url::parse(&format!("{}/get", main_url))?,
        login_url: Url::parse(&format!("{}/login", main_url))?,
        create_account_url: Url::parse(&format!("{}/create", main_url))?,
        list_url: Url::parse(&format!("{}/list", main_url))?,
    };

    let key_path = Path::new("test-rsa-key.pk8");
    let keypair_unchecked = crypto::get_key_pair(key_path);
    if keypair_unchecked.is_err() {
        println!(
            "Error getting keypair at {}",
            key_path.as_os_str().to_str().unwrap_or_else(|| "ERROR")
        );
        exit(1);
    }
    let keypair = keypair_unchecked.unwrap();

    let mut buffer = String::new();
    //let mut username = String::from("admin");

    let userpassword = String::from("hunter42");

    // password is used as salt and cant be shorter than 8 characters
    if userpassword.len() < 8 {
        println!("Too short password");
        exit(1);
    }

    /*println!("{}", format!("Login or Create account at {}", main_url));

    let mut username = String::new();
    let mut userpassword = String::new();
    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer)?;

        match buffer.trim().split_once(" ") {
            Some((prefix, data)) => match prefix {
                "login" => {
                    if let Some((name, psw)) = data.split_once(" ") {
                        match site.login(name.to_string(), psw.to_string()).await {
                            Ok(status) => match status {
                                LoginStatus::Success => {
                                    username = name.to_string();
                                    userpassword = psw.to_string();
                                    println!("Login successful");
                                    break;
                                }
                                LoginStatus::WrongPassword => println!("Wrong password"),
                                LoginStatus::AccountNotFound => println!("Account Not Found"),
                                LoginStatus::Error => println!("Server error"),
                            },
                            Err(msg) => println!("{}", msg),
                        }
                    } else {
                        println!("Invalid username/password input");
                    }
                }
                "create" => {
                    if let Some((name, psw)) = data.split_once(" ") {
                        match site.create(name.to_string(), psw.to_string()).await {
                            Ok(status) => match status {
                                CreateStatus::Success => {
                                    username = name.to_string();
                                    userpassword = psw.to_string();
                                    println!("Login successful");
                                    break;
                                }
                                CreateStatus::AccountTaken => println!("Account already taken"),
                                CreateStatus::Error => println!("Server error"),
                            },
                            Err(msg) => println!("{}", msg),
                        }
                    } else {
                        println!("Invalid username/password input");
                    }
                }
                _ => {
                    println!("Invalid prefix")
                }
            },
            _ => match &buffer.trim()[0..] {
                "exit" | "quit" | "q" => {
                    exit(0);
                }
                _ => {
                    println!("Invalid login");
                }
            },
        }
    }*/

    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer)?;

        match buffer.trim().split_once(" ") {
            Some((prefix, data)) => match prefix {
                "pull" => {
                    if let Err(msg) = site
                        .pull_file(
                            data.to_string(),
                            //username.clone(),
                            userpassword.clone(),
                            &keypair,
                        )
                        .await
                    {
                        println!("{}", msg)
                    }
                }
                "push" => {
                    if let Err(msg) = site
                        .push_file(
                            Path::new(data.trim()),
                            //username.clone(),
                            userpassword.clone(),
                            &keypair,
                        )
                        .await
                    {
                        println!("{}", msg)
                    }
                }
                _ => {
                    println!("Invalid prefix")
                }
            },
            _ => match &buffer.trim()[0..] {
                "list" => {
                    if let Err(msg) = site.list_files(userpassword.clone()).await {
                        println!("{}", msg)
                    }
                }
                "exit" | "quit" | "q" => {
                    exit(0);
                }
                _ => {
                    println!("Invalid input");
                }
            },
        }
    }

    Ok(())
}
