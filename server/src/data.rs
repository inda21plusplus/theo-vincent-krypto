use ring::digest::{digest, Digest, SHA256};
use std::collections::HashMap;
use types::{FileData as NetworkFileData, FileInfo};

use std::io::{self, prelude::*};

use super::file::File;
use super::merkle_tree::MerkleTree;

#[derive(Debug)]
pub struct Files {
    file_id: u64,
    tree: MerkleTree,
    file_map: HashMap<String, u64>,
}

impl Files {
    pub fn new() -> Self {
        Files {
            file_id: 0,
            tree: MerkleTree::new(),
            file_map: HashMap::new(),
        }
    }

    pub fn get_all_files(&self) -> Vec<(&String, &File)> {
        let mut list = vec![];
        for (k, v) in &self.file_map {
            if let Some(file) = self.tree.get_file(*v) {
                list.push((k, file));
            }
        }
        list
    }

    pub fn top_hash(&self) -> &Digest {
        self.tree.top_hash()
    }

    pub fn add_file(&mut self, data: NetworkFileData) {
        let id = self.file_id;
        self.file_id += 1;
        self.file_map.insert(data.name_hash.clone(), id);
        let node = self.tree.get_file_mut(id);
        *node = Some(File::new(data));
        self.tree.recompute_hashes();
    }

    pub fn get_file(&mut self, info: FileInfo) -> Option<NetworkFileData> {
        let id = match self.file_map.get(&info.name_hash) {
            Some(x) => {
                println!("Requested ID wasn't found (id: {})", info.name_hash);
                *x
            }
            None => return None,
        };

        let f = self
            .tree
            .get_file(id)
            .as_ref()
            .map(|x| x.file_data().unwrap());

        if f.is_none() {
            println!("File was empty (id: {})", info.name_hash);
        }

        f
    }

    pub fn get_merkle_data(&mut self, name: &str) -> Option<types::MerkleData> {
        let id = match self.file_map.get(name) {
            Some(x) => {
                println!("Requested ID wasn't found (id: {})", name);
                *x
            }
            None => return None,
        };

        Some(self.tree.get_merkle_data_for_file(id))
    }
}
