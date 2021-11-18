use serde::{Deserialize, Serialize};

type Hash = Vec<u8>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileData {
    pub name_nonce: [u8; 12],
    pub name: Vec<u8>,     // used for client to read the name of the file
    pub name_hash: String, // used to look up the file
    pub nonce: [u8; 12],
    pub contents: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateInfo {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginInfo {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileList {
    pub top_hash: Hash,
    pub list: Vec<FileListEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileListEntry {
    pub name_hash: String,
    pub nonce: [u8; 12],
    pub name: Vec<u8>,
    pub name_nonce: [u8; 12],
    pub size: usize,
}

/// All the neighboring hashes required to compute a new top hash.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleData {
    pub top_hash: Hash,
    pub hashes: Vec<(Side, Hash)>,
}

/// The side to append the given hash to
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
