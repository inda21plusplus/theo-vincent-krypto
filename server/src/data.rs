use argon2::{self, Config};
use lockfree::map::Map;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use types::FileData;

#[derive(Debug)]
pub struct User {
    name: String,
    passwd: String,
    files: Files,
}

#[derive(Debug, Default)]
pub struct Files {
    files: Vec<ServerFile>,
}

#[derive(Debug)]
pub enum ServerFile {
    Persistent,
    Ephemeral(FileData),
}

pub struct Database {
    list: Map<String, RwLock<User>>,
}

impl Database {
    pub fn new() -> Self {
        Self { list: Map::new() }
    }

    pub fn create_user(&self, name: String, passwd: String) -> bool {
        if self.list.get(&name).is_some() {
            false
        } else {
            let user = User {
                name: name.clone(),
                passwd,
                files: Files::default(),
            };

            let res = self.list.insert(name, RwLock::new(user));

            assert!(res.is_none());

            true
        }
    }

    pub fn authenticate(&self, name: &str, passwd: &str) -> AccountStatus {
        if let Some(user) = self.list.get(&name.to_string()) {
            if &user.val().read().unwrap().passwd == passwd {
                AccountStatus::Success
            } else {
                AccountStatus::WrongPassword
            }
        } else {
            AccountStatus::NonExistent
        }
    }
}

pub enum AccountStatus {
    WrongPassword,
    NonExistent,
    Success,
}
