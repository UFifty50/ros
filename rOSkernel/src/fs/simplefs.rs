// use alloc::{boxed::Box, string::String, sync::Arc, vec};

// use crate::fs::vfs::{FileStat, FileType, FsError, FsResult, INode};

// const SFS_MAGIC: [u8; 3] = [0x53, 0x46, 0x53]; // "SFS"
// const SFS_VERSION_1_10: u8 = 0x1A;

// // Entry Types
// const SFS_ENTRY_VOL_ID: u8 = 0x01;
// const SFS_ENTRY_START: u8 = 0x02;
// const SFS_ENTRY_UNUSED: u8 = 0x10;
// const SFS_ENTRY_DIR: u8 = 0x11;
// const SFS_ENTRY_FILE: u8 = 0x12;

// #[derive(Debug, Clone)]
// struct SfsSuperBlock {
//     timestamp: i64,
//     data_size: u64,  // in blocks
//     index_size: u64, // in bytes
//     total_blocks: u64,
//     rsvd_blocks: u32,
//     block_size_log: u8, // log2(block_size) - 7. e.g. 2 => 512 bytes
// }

// impl SfsSuperBlock {
//     fn block_size(&self) -> u64 {
//         1u64 << (self.block_size_log + 7)
//     }

//     fn parse(data: &[u8]) -> FsResult<Self> {
//         // Offset 0x18E in the first block
//         if data.len() < 0x18E + size_of::<SelfRaw>() {
//             return Err(FsError::IoError);
//         }
//         let raw_slice = &data[0x18E..0x18E + 30]; // Struct is roughly 30 bytes
        
//         // Manual Little Endian parsing
//         let magic = &raw_slice[20..23];
//         if magic != SFS_MAGIC {
//             return Err(FsError::IoError); // Invalid Magic
//         }

//         let sb = Self {
//             timestamp: i64::from_le_bytes(raw_slice[0..8].try_into().unwrap()),
//             data_size: u64::from_le_bytes(raw_slice[8..16].try_into().unwrap()),
//             index_size: u64::from_le_bytes(raw_slice[16..24].try_into().unwrap()),
//             // magic [20..23]
//             // version [23]
//             total_blocks: u64::from_le_bytes(raw_slice[24..32].try_into().unwrap()),
//             rsvd_blocks: u32::from_le_bytes(raw_slice[32..36].try_into().unwrap()),
//             block_size_log: raw_slice[36],
//         };
        
//         Ok(sb)
//     }
// }

// // Just for sizing reference, we parse manually
// struct SelfRaw { /* defined implicitly in parse */ }

// // ==========================================
// // 3. The Driver INode Implementation
// // ==========================================

// /// The INode struct that implements the VFS trait.
// /// 
// /// SFS is stateless regarding the graph (it has no tree on disk).
// /// Therefore, this struct holds the "Logical Path" it represents.
// /// When lookup() is called, it appends the child name to its own path
// /// and scans the disk for that string.
// pub struct SfsINode {
//     device: Arc<dyn BlockDevice>,
//     sb: Arc<SfsSuperBlock>,
//     /// The full path of this node (e.g., "home/user"). No leading slash.
//     path: String,
//     /// Cached metadata from the last disk read
//     meta: FileStat,
//     /// Disk location info (needed for read/write)
//     start_block: u64,
//     file_length: u64, 
// }

// impl SfsINode {
//     /// Create the Root INode
//     pub fn new_root(device: Arc<dyn BlockDevice>) -> FsResult<Box<dyn INode>> {
//         // 1. Read Superblock (LBN 0)
//         let mut buf = vec![0u8; device.sector_size() as usize];
//         device.read_sector(0, &mut buf)?;
        
//         let sb = Arc::new(SfsSuperBlock::parse(&buf)?);
        
//         Ok(Box::new(Self {
//             device,
//             sb,
//             path: String::new(), // Root is empty string in SFS path logic
//             meta: FileStat {
//                 fileType: FileType::Directory,
//                 size: 0,
//                 blockSize: 512, // default, updated later
//                 blocks: 0,
//             },
//             start_block: 0,
//             file_length: 0,
//         }))
//     }

//     /// Internal helper to scan the Index Area
//     fn find_entry(&self, target_path: &str) -> FsResult<EntryInfo> {
//         let block_size = self.sb.block_size();
//         let index_size = self.sb.index_size;
//         let total_blocks = self.sb.total_blocks;

//         // Calculate byte range of Index Area
//         let end_offset = total_blocks * block_size;
//         let start_offset = end_offset - index_size;

//         // Determine blocks to read. 
//         // SFS Index grows backwards from end of volume.
//         // We will read block by block from End -> Start (or Start -> End).
//         // Spec: "First entry is at the end of the last block... sequential entry toward start."
//         // This means physical entry 0 is at the very end.
        
//         let mut current_offset = end_offset;
//         let mut buffer = vec![0u8; block_size as usize];
        
//         // We iterate backwards through the index area
//         while current_offset > start_offset {
//             current_offset -= 64; // Size of one entry
            
//             // Logic to read the sector containing 'current_offset'
//             // Simplified: assuming block_size == sector_size for this snippet.
//             // In reality, map offset -> LBN -> LBA.
//             let lbn = current_offset / block_size;
//             let offset_in_block = (current_offset % block_size) as usize;
            
//             self.device.read_sector(lbn, &mut buffer)?;
//             let entry_raw = &buffer[offset_in_block..offset_in_block + 64];
            
//             let entry_type = entry_raw[0];

//             // Skip unused/deleted/special
//             if entry_type != SFS_ENTRY_FILE && entry_type != SFS_ENTRY_DIR {
//                 continue;
//             }

//             // Parse Name (handle continuations if necessary)
//             // Note: This is a simplified read. Full SFS requires reading previous entries
//             // if num_cont > 0.
//             let name = self.parse_name(entry_raw, current_offset)?;

//             if name == target_path {
//                 return self.parse_entry_info(entry_raw, entry_type);
//             }
//         }

//         Err(FsError::NotFound)
//     }

//     /// Helper to extract name from entry (handling continuations omitted for brevity)
//     fn parse_name(&self, raw: &[u8], _offset: u64) -> FsResult<String> {
//         // offset of name in File Entry is 28, Dir Entry is 11.
//         let type_ = raw[0];
//         let name_offset = if type_ == SFS_ENTRY_FILE { 35 } else { 11 };
//         let name_len = if type_ == SFS_ENTRY_FILE { 29 } else { 53 };
        
//         let name_slice = &raw[name_offset..name_offset+name_len];
        
//         // Find null terminator
//         let len = name_slice.iter().position(|&c| c == 0).unwrap_or(name_len);
//         let s = String::from_utf8(name_slice[0..len].to_vec()).map_err(|_| FsError::IoError)?;
//         Ok(s)
//     }

//     fn parse_entry_info(&self, raw: &[u8], type_: u8) -> FsResult<EntryInfo> {
//         let is_dir = type_ == SFS_ENTRY_DIR;
        
//         let (size, start, _end) = if is_dir {
//             (0, 0, 0)
//         } else {
//             let start = u64::from_le_bytes(raw[11..19].try_into().unwrap());
//             let end = u64::from_le_bytes(raw[19..27].try_into().unwrap());
//             let size = u64::from_le_bytes(raw[27..35].try_into().unwrap());
//             (size, start, end)
//         };

//         Ok(EntryInfo {
//             type_: if is_dir { FileType::Directory } else { FileType::RegularFile },
//             size,
//             start_block: start,
//         })
//     }
// }

// struct EntryInfo {
//     type_: FileType,
//     size: u64,
//     start_block: u64,
// }

// impl INode for SfsINode {
//     fn stat(&self) -> FsResult<FileStat> {
//         Ok(self.meta.clone())
//     }

//     fn lookup(&self, name: &str) -> FsResult<Box<dyn INode>> {
//         // 1. Construct the target path
//         // SFS uses "dir/file" format. No leading slash.
//         let target_path = if self.path.is_empty() {
//             String::from(name)
//         } else {
//             alloc::format!("{}/{}", self.path, name)
//         };

//         // 2. Scan the disk
//         let info = self.find_entry(&target_path)?;

//         // 3. Return new INode
//         Ok(Box::new(SfsINode {
//             device: self.device.clone(),
//             sb: self.sb.clone(),
//             path: target_path,
//             meta: FileStat {
//                 fileType: info.type_,
//                 size: info.size,
//                 blockSize: self.sb.block_size() as u32,
//                 blocks: (info.size + self.sb.block_size() - 1) / self.sb.block_size(),
//             },
//             start_block: info.start_block,
//             file_length: info.size,
//         }))
//     }

//     fn create(&self, name: &str, kind: FileType) -> FsResult<Box<dyn INode>> {
//         if kind == FileType::Directory {
//             return self.mkdir(name);
//         }
        
//         // 1. Check if exists
//         let target_path = if self.path.is_empty() {
//             String::from(name)
//         } else {
//             alloc::format!("{}/{}", self.path, name)
//         };
        
//         if let Ok(_) = self.find_entry(&target_path) {
//             return Err(FsError::AlreadyExists);
//         }

//         // 2. SFS Create Logic (Summary):
//         // a. Find space in Data Area (append to end of used data).
//         // b. Find unused entry in Index Area.
//         // c. Write File Entry with `target_path`.
        
//         // NOTE: Full write implementation is complex due to block allocation
//         // and index shifting. Returning NotSupported for brevity of the example,
//         // but this is where you'd implement the "write entry" logic.
//         Err(FsError::NotSupported)
//     }

//     fn mkdir(&self, name: &str) -> FsResult<Box<dyn INode>> {
//         // Similar to create, but writes a Dir Entry (Type 0x11)
//         // Dir entries have no data blocks, just the name entry in the index.
//         Err(FsError::NotSupported)
//     }

//     fn rmdir(&self, _name: &str) -> FsResult<()> {
//         // Find entry, change type to SFS_ENTRY_DIR_DEL (0x19)
//         Err(FsError::NotSupported)
//     }

//     fn rename(&self, _oldname: &str, _newname: &str) -> FsResult<()> {
//         // Find entry, rewrite name field.
//         // If new name is longer, might need to reallocate continuation entries.
//         Err(FsError::NotSupported)
//     }

//     fn link(&self, _name: &str, _target: &dyn INode) -> FsResult<()> {
//         // SFS does not support hard links.
//         // It is a flat list of files.
//         Err(FsError::NotSupported)
//     }

//     fn unlink(&self, _name: &str) -> FsResult<()> {
//         // Find entry, change type to SFS_ENTRY_FILE_DEL (0x1A)
//         Err(FsError::NotSupported)
//     }

//     fn symlink(&self, _name: &str, _target: &str) -> FsResult<()> {
//         // SFS spec does not mention symlinks.
//         Err(FsError::NotSupported)
//     }
// }
