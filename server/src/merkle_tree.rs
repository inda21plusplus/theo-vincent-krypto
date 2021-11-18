const MAX_DEPTH: u64 = 8;
const MAX_FILE_ID: u64 = 1 << MAX_DEPTH;

use serde::{Deserialize, Serialize};

use crate::file::File;
use ring::digest::{digest, Digest, SHA256};
use types::Side;

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

    pub fn top_hash(&self) -> &Digest {
        self.root.digest()
    }

    fn rec_create_nodes(depth: u64, id: u64) -> Node {
        if depth == 0 {
            Node::Leaf {
                id: id,
                hash: digest(&SHA256, &[]),
                data: None,
                dirty: false,
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
                dirty: false,
            }
        }
    }

    pub fn get_root_hash(&self) -> &[u8] {
        self.root.hash_bytes()
    }

    pub fn get_file(&self, id: u64) -> &Option<File> {
        self.root.get_file(id)
    }

    pub fn get_file_mut(&mut self, id: u64) -> &mut Option<File> {
        self.root.get_file_mut(id)
    }

    pub fn recompute_hashes(&mut self) {
        self.root.recompute_hash_if_dirty()
    }

    pub fn get_merkle_data_for_file(&mut self, id: u64) -> types::MerkleData {
        let mut v = Vec::new();
        self.root.get_hashes_for_file(id, &mut v);
        let v = v
            .into_iter()
            .map(|(side, dig)| (side, dig.as_ref().to_vec()))
            .collect::<Vec<_>>();
        types::MerkleData {
            top_hash: self.root.digest().as_ref().to_vec(),
            hashes: v,
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Leaf {
        id: u64,
        data: Option<File>,
        hash: Digest,
        dirty: bool,
    },
    Branch {
        hash: Digest,
        left: Box<Node>,
        right: Box<Node>,
        dirty: bool,
    },
}

impl Node {
    /// Retreive a mutable handle to a file from the merkle tree using
    /// the ID, marking every branch as dirty along the way
    pub fn get_file_mut(&mut self, id: u64) -> &mut Option<File> {
        match self {
            Node::Branch {
                left,
                right,
                ref mut dirty,
                ..
            } => {
                *dirty = true;
                match id & 1 {
                    0 => left,
                    1 => right,
                    _ => unreachable!(),
                }
                .get_file_mut(id >> 1)
            }
            Node::Leaf {
                ref mut data,
                ref mut dirty,
                ..
            } => {
                *dirty = true;
                data
            }
        }
    }

    pub fn get_file(&self, id: u64) -> &Option<File> {
        match self {
            Node::Branch { left, right, .. } => match id & 1 {
                0 => left,
                1 => right,
                _ => unreachable!(),
            }
            .get_file(id >> 1),
            Node::Leaf { data, .. } => data,
        }
    }

    /// Recomputes the hash for all nodes marked as dirty
    pub fn recompute_hash_if_dirty(&mut self) {
        if *self.dirty_mut() {
            *self.dirty_mut() = false;

            match self {
                Node::Leaf { hash, data, .. } => {
                    *hash = digest(
                        &SHA256,
                        &data
                            .as_ref()
                            .map(|x| x.contents().unwrap())
                            .unwrap_or(vec![]),
                    );
                }
                Node::Branch {
                    hash, left, right, ..
                } => {
                    left.recompute_hash_if_dirty();
                    right.recompute_hash_if_dirty();
                    let mut concat = left.hash_bytes().to_vec();
                    concat.extend_from_slice(&right.hash_bytes());
                    *hash = digest(&SHA256, &concat[..]);
                }
            }
        }
    }

    pub fn dirty_mut(&mut self) -> &mut bool {
        match self {
            Node::Leaf { dirty, .. } => dirty,
            Node::Branch { dirty, .. } => dirty,
        }
    }

    /// Unconditionally recomputes the hashes for all nodes
    pub fn recompute_hash_full(&mut self) {
        match self {
            Node::Leaf { hash, data, .. } => {
                if let Some(file) = data {
                    *hash = digest(&SHA256, &file.contents().unwrap());
                } else {
                    *hash = digest(&SHA256, &[]);
                }
            }
            Node::Branch {
                hash, left, right, ..
            } => {
                left.recompute_hash_full();
                right.recompute_hash_full();
                let mut concat = left.hash_bytes().to_vec();
                concat.extend_from_slice(&right.hash_bytes());
                *hash = digest(&SHA256, &concat[..]);
            }
        }
        *self.dirty_mut() = false;
    }

    pub fn digest(&self) -> &Digest {
        match self {
            Node::Leaf { hash, .. } => hash,
            Node::Branch { hash, .. } => hash,
        }
    }

    pub fn hash_bytes(&self) -> &[u8] {
        match self {
            Node::Leaf { hash, .. } => hash,
            Node::Branch { hash, .. } => hash,
        }
        .as_ref()
    }

    pub fn get_hashes_for_file(&self, id: u64, v: &mut Vec<(Side, Digest)>) {
        match self {
            Node::Branch { left, right, .. } => match id & 1 {
                0 => {
                    v.push((Side::Right, right.digest().clone()));
                    left.get_hashes_for_file(id >> 1, v)
                }
                1 => {
                    v.push((Side::Left, left.digest().clone()));
                    right.get_hashes_for_file(id >> 1, v)
                }
                _ => unreachable!(),
            },
            Node::Leaf { .. } => {}
        }
    }
}

impl Default for MerkleTree {
    fn default() -> MerkleTree {
        MerkleTree::new()
    }
}
