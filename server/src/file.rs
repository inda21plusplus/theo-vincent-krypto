use serde::{Deserialize, Serialize};

use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{self, prelude::*};
use std::ops::Deref;
use std::path::{Path, PathBuf};

static SAVE_DIR_VAR: &'static str = "SERVER_SAVE_DIR";

use types::FileData as MemoryFile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RawFile {
    Memory(MemoryFile),
    Disk(PersistentFile),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    inner: RefCell<RawFile>,
    size: usize,
}

impl File {
    pub fn new(data: MemoryFile) -> Self {
        let size = data.contents.len();
        Self {
            inner: RefCell::new(RawFile::Memory(data)),
            size,
        }
    }

    pub fn file_data(&self) -> io::Result<MemoryFile> {
        let mut borrow = self.inner.borrow_mut();
        borrow.load()?;
        Ok(match borrow.clone() {
            RawFile::Memory(x) => x,
            _ => unreachable!(),
        })
    }

    pub fn file_data_mut(&mut self) -> &mut MemoryFile {
        todo!()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn contents(&self) -> io::Result<Vec<u8>> {
        Ok(self.inner.borrow_mut().load()?.contents.clone())
    }

    pub fn name_hash(&self) -> String {
        match self.inner.borrow().deref() {
            RawFile::Disk(pf) => pf.name_hash.clone(),
            RawFile::Memory(mf) => mf.name_hash.clone(),
        }
    }

    pub fn name_nonce(&self) -> [u8; 12] {
        match self.inner.borrow().deref() {
            RawFile::Disk(pf) => pf.name_nonce,
            RawFile::Memory(mf) => mf.name_nonce,
        }
    }

    pub fn name(&self) -> Vec<u8> {
        match self.inner.borrow().deref() {
            RawFile::Disk(pf) => pf.name.clone(),
            RawFile::Memory(mf) => mf.name.clone(),
        }
    }

    pub fn nonce(&self) -> [u8; 12] {
        match self.inner.borrow().deref() {
            RawFile::Disk(pf) => pf.nonce,
            RawFile::Memory(mf) => mf.nonce,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentFile {
    pub name_nonce: [u8; 12],
    pub name: Vec<u8>,
    pub name_hash: String,
    pub path: PathBuf,
    pub nonce: [u8; 12],
    pub signature: Vec<u8>,
}

impl RawFile {
    fn load(&mut self) -> io::Result<&MemoryFile> {
        let pf = match self {
            RawFile::Memory(mf) => return Ok(&*mf),
            RawFile::Disk(pf) => pf,
        };

        let PersistentFile {
            name_nonce,
            name,
            name_hash,
            path,
            nonce,
            signature,
        } = pf.clone();

        let mut buf = Vec::new();
        let mut f = fs::File::open(&path)?;
        f.read_to_end(&mut buf)?;

        *self = RawFile::Memory(MemoryFile {
            name_nonce,
            name,
            name_hash,
            nonce,
            signature,
            contents: buf,
        });

        match self {
            RawFile::Memory(mf) => Ok(&*mf),
            _ => unreachable!(),
        }
    }
    fn save(&mut self) -> io::Result<()> {
        let mf = match self {
            RawFile::Disk(_) => return Ok(()),
            RawFile::Memory(mf) => mf,
        };

        let mut path = match env::var_os(SAVE_DIR_VAR) {
            Some(dir) => Path::new(&dir).to_path_buf(),
            None => env::temp_dir(),
        };

        path.push(&mf.name_hash);

        let contents = std::mem::replace(&mut mf.contents, vec![]);

        let mut f = fs::File::create(&path)?;
        f.write_all(&contents[..])?;

        let MemoryFile {
            name_nonce,
            name,
            name_hash,
            nonce,
            signature,
            ..
        } = mf.clone();

        *self = RawFile::Disk(PersistentFile {
            name_nonce,
            name,
            name_hash,
            path,
            nonce,
            signature,
        });

        Ok(())
    }
}
