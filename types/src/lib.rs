use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileData {    
    pub name_nonce: [u8; 12],
    pub name: Vec<u8>, // used for client to read the name of the file
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
