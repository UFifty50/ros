// use super::vfs::{OpenFlags, SeekFrom, FsResult, FsError, INode};
// use alloc::{sync::Arc, vec::Vec};
// use spin::Mutex;

// /// An open file handle
// pub struct OpenFile {
//     pub inode: Arc<dyn INode>,
//     pub flags: OpenFlags,
//     pub offset: Mutex<u64>,
// }

// impl OpenFile {
//     pub fn new(inode: Arc<dyn INode>, flags: OpenFlags) -> Self {
//         Self {
//             inode,
//             flags,
//             offset: Mutex::new(0),
//         }
//     }
    
//     pub fn read(&self, buf: &mut [u8]) -> FsResult<usize> {
//         if !self.flags.contains(OpenFlags::READ) {
//             return Err(FsError::PermissionDenied);
//         }
//         let mut offset = self.offset.lock();
//         let bytes_read = self.vnode.read_at(*offset, buf)?;
//         *offset += bytes_read as u64;
//         Ok(bytes_read)
//     }
    
//     pub fn write(&self, buf: &[u8]) -> FsResult<usize> {
//         if !self.flags.contains(OpenFlags::WRITE) {
//             return Err(FsError::PermissionDenied);
//         }
//         let mut offset = self.offset.lock();
//         if self.flags.contains(OpenFlags::APPEND) {
//             *offset = self.vnode.stat()?.size;
//         }
//         let bytes_written = self.vnode.write_at(*offset, buf)?;
//         *offset += bytes_written as u64;
//         Ok(bytes_written)
//     }
    
//     pub fn seek(&self, pos: SeekFrom) -> FsResult<u64> {
//         let mut offset = self.offset.lock();
//         let size = self.vnode.stat()?.size;
        
//         let new_offset = match pos {
//             SeekFrom::Start(n) => n,
//             SeekFrom::Current(n) => (*offset as i64 + n) as u64,
//             SeekFrom::End(n) => (size as i64 + n) as u64,
//         };
        
//         *offset = new_offset;
//         Ok(new_offset)
//     }
// }

// /// File descriptor table (one per process)
// pub struct FdTable {
//     files: Vec<Option<Arc<OpenFile>>>,
// }

// impl FdTable {
//     pub fn new() -> Self {
//         // Reserve fd 0, 1, 2 for stdin, stdout, stderr
//         Self {
//             files: Vec::new(),
//         }
//     }
    
//     /// Allocate a new file descriptor
//     pub fn insert(&mut self, file: Arc<OpenFile>) -> usize {
//         // Find first empty slot
//         for (i, slot) in self.files.iter_mut().enumerate() {
//             if slot.is_none() {
//                 *slot = Some(file);
//                 return i;
//             }
//         }
//         // No empty slot, push new
//         let fd = self.files.len();
//         self.files.push(Some(file));
//         fd
//     }
    
//     pub fn get(&self, fd: usize) -> Option<Arc<OpenFile>> {
//         self.files.get(fd)?.clone()
//     }
    
//     pub fn close(&mut self, fd: usize) -> FsResult<()> {
//         match self.files.get_mut(fd) {
//             Some(slot @ Some(_)) => {
//                 *slot = None;
//                 Ok(())
//             }
//             _ => Err(FsError::NotFound),
//         }
//     }
// }
