use ring::digest::{digest, Digest, SHA256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use types::{FileData, FileInfo};

const MAX_DEPTH: u64 = 8;
const MAX_FILE_ID: u64 = 1 << MAX_DEPTH;

#[derive(Debug, Default)]
pub struct Files {
    file_id: u64,
    tree: MerkleTree,
    file_map: HashMap<String, u64>,
}

impl Files {
    pub fn new() {}

    pub fn add_file(&mut self, data: FileData) {
        let id = self.file_id;
        self.file_id += 1;
        self.file_map.insert(data.name_hash.clone(), id);
        let node = self.tree.get_file(id);
        *node = Some(data);
    }

    pub fn get_file(&mut self, info: FileInfo) -> &Option<FileData> {
        let id = self.file_map.get(&info.name_hash);
        &*self.tree.get_file(match id {
            Some(x) => *x,
            None => return &None,
        })
    }
}

#[derive(Debug)]
pub struct MerkleTree {
    root: Node,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            root: Self::rec_create_nodes(MAX_DEPTH, 0),
        }
    }

    fn rec_create_nodes(depth: u64, id: u64) -> Node {
        if depth == 0 {
            Node::Leaf {
                id: id,
                hash: digest(&SHA256, &[]),
                data: None,
            }
        } else {
            let left = Box::new(Self::rec_create_nodes(depth - 1, id << 1));
            let right = Box::new(Self::rec_create_nodes(depth - 1, (id << 1) | 1));

            let mut combined = left.hash_bytes().to_vec();
            combined.extend_from_slice(right.hash_bytes());

            Node::Branch {
                hash: digest(&SHA256, &combined),
                left,
                right,
            }
        }
    }

    pub fn get_root_hash(&self) -> &[u8] {
        self.root.hash_bytes()
    }

    pub fn get_file(&mut self, id: u64) -> &mut Option<FileData> {
        self.root.get_file(id)
    }
}

#[derive(Debug)]
pub enum Node {
    Leaf {
        id: u64,
        data: Option<FileData>,
        hash: Digest,
    },
    Branch {
        hash: Digest,
        left: Box<Node>,
        right: Box<Node>,
    },
}

const LEFT: u64 = 0;
const RIGHT: u64 = 1;

impl Node {
    pub fn get_file(&mut self, id: u64) -> &mut Option<FileData> {
        match self {
            Node::Branch { left, right, .. } => match id & 1 {
                0 => left,
                1 => right,
                _ => unreachable!(),
            }
            .get_file(id >> 1),
            Node::Leaf { ref mut data, .. } => data,
        }
    }

    pub fn hash_bytes(&self) -> &[u8] {
        match self {
            Node::Leaf { hash, .. } => hash,
            Node::Branch { hash, .. } => hash,
        }
        .as_ref()
    }
}

impl Default for MerkleTree {
    fn default() -> MerkleTree {
        MerkleTree::new()
    }
}
