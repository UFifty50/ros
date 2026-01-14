use alloc::{boxed::Box, string::String, sync::Arc};
use hashbrown::HashMap;

/// Errors that can occur during filesystem operations
#[derive(Debug, Clone, Copy)]
pub enum FsError {
    NotFound,
    PermissionDenied,
    NotADirectory,
    IsADirectory,
    AlreadyExists,
    NotEmpty,
    InvalidPath,
    NoSpace,
    IoError,
    NotSupported,
}

pub type FsResult<T> = Result<T, FsError>;

/// File types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    RegularFile,
    Directory,
    // CharDevice,
    // BlockDevice,
    // Symlink,
    // Socket,
    // Fifo,
}

/// File metadata
#[derive(Debug, Clone)]
pub struct FileStat {
    pub fileType: FileType,
    pub size: u64,
    pub blockSize: u32,
    pub blocks: u64,
    // Permissions, timestamps, etc. can be added later
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct OpenFlags: u32 {
        const READ      = 0b0000_0001;
        const WRITE     = 0b0000_0010;
        const APPEND    = 0b0000_0100;
        const CREATE    = 0b0000_1000;
        const TRUNCATE  = 0b0001_0000;
        const EXCLUSIVE = 0b0010_0000;
    }
}

/// Seek origin for lseek
#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(u64),
    Current(i64),
    End(i64),
}

pub struct TNode {
    pub name: String,
    pub vinode: Arc<VINode>,
}

pub enum VINode {
    File(VFileINode),
    Folder(VFolderINode),
}

pub struct VFolderINode {
    pub entries: HashMap<String, Arc<TNode>>,
    pub driverINode: Box<dyn INode>,
}

pub struct VFileINode {
    //TODO: other metadata
    pub created: u64,
    pub modified: u64,
    pub size: u64,
    pub driverINode: Box<dyn INode>,
}

pub trait INode: Send + Sync {
    fn stat(&self) -> FsResult<FileStat>;
    fn lookup(&self, name: &str) -> FsResult<Box<dyn INode>>;
    fn create(&self, name: &str, kind: FileType) -> FsResult<Box<dyn INode>>;
    fn mkdir(&self, name: &str) -> FsResult<Box<dyn INode>>;
    fn rmdir(&self, name: &str) -> FsResult<()>;
    fn rename(&self, oldname: &str, newname: &str) -> FsResult<()>;
    fn link(&self, name: &str, target: &dyn INode) -> FsResult<()>;
    fn unlink(&self, name: &str) -> FsResult<()>;
    fn symlink(&self, name: &str, target: &str) -> FsResult<()>;
}
