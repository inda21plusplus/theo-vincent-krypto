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
    pub name: String,
    pub size: usize,
}

/// All the neighboring hashes required to compute a new top hash, the
/// final one being the new top hash. Note, the server's newly
/// computed file hash is not included.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleInfo {
    hashes: Vec<Hash>,
}
