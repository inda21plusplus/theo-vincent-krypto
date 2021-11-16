use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileData {
    name: String,
    contents: Vec<u8>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
