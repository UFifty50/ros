#!/usr/bin/env python3
import sys
import os
import struct
import time
import math
import argparse
from typing import Tuple

# --- Constants & Spec Definitions ---
BLOCK_SIZE_EXP = 2  # 2^9 = 512 bytes
MAGIC = b'SFS'
VERSION = 0x1A

# Entry Types
TYPE_VOL_ID     = 0x01
TYPE_START      = 0x02
TYPE_UNUSED     = 0x10
TYPE_DIR        = 0x11
TYPE_FILE       = 0x12
TYPE_UNUSABLE   = 0x18
TYPE_DIR_DEL    = 0x19
TYPE_FILE_DEL   = 0x1A

ENTRY_SIZE = 64
SUPER_OFFSET = 0x18E
SUPER_SIZE = 42 
FILE_NAME_LEN = 29
DIR_NAME_LEN = 53
FMT_SUPER = "<qQQ3sBQIBB"

def get_time_stamp(): return int(time.time() * 65536)

def calc_crc(data: bytes) -> int:
    total = sum(data)
    return (256 - (total & 0xFF)) & 0xFF

def validate_crc(data: bytes) -> bool: return (sum(data) & 0xFF) == 0

class SFSEntry:
    def __init__(self, raw_data=None):
        self.raw = bytearray(raw_data) if raw_data else bytearray(ENTRY_SIZE)
        self.is_valid = False
        self.entry_type = 0
        self.continuations = [] 
        if raw_data: self.parse()

    def parse(self):
        self.entry_type = self.raw[0]
        if not validate_crc(self.raw): return 
        self.is_valid = True

    @property
    def num_cont(self):
        if self.entry_type in [TYPE_FILE, TYPE_DIR, TYPE_FILE_DEL, TYPE_DIR_DEL]: return self.raw[2]
        return 0

    def get_name(self):
        name_bytes = b""
        if self.entry_type in [TYPE_DIR, TYPE_DIR_DEL]: name_bytes = self.raw[11:11+DIR_NAME_LEN]
        elif self.entry_type in [TYPE_FILE, TYPE_FILE_DEL]: name_bytes = self.raw[35:35+FILE_NAME_LEN]
        elif self.entry_type == TYPE_VOL_ID: name_bytes = self.raw[12:12+52]
        for cont in self.continuations: name_bytes += cont
        try: return name_bytes.split(b'\x00')[0].decode('utf-8')
        except: return "<Invalid Name>"
            
    def update_crc(self):
        self.raw[1] = 0
        total_data = self.raw + b"".join(self.continuations)
        self.raw[1] = calc_crc(total_data)

class SFSVolume:
    def __init__(self, path, mode='rb+', init_size=1024*1024):
        self.path = path
        self.mode = mode
        self.handle = None
        self.super = {}
        self.entries = [] 
        
        if 'w' in mode:
             self.handle = open(path, 'wb+')
             self.format_new(size_bytes=init_size)
             return

        if os.path.exists(path):
            self.handle = open(path, mode)
            try:
                self.read_super()
                self.block_size = 1 << (self.super['block_size'] + 7)
            except Exception as e:
                 print(f"Error reading SFS volume: {e}")
                 sys.exit(1)
        else:
            if 'a' in mode:
                self.handle = open(path, 'wb+')
                self.format_new(size_bytes=init_size)
            else:
                print("File not found.")
                sys.exit(1)

    def format_new(self, size_bytes=1024*1024):
        self.handle.truncate(size_bytes)
        self.block_size = 512
        total_blocks = size_bytes // self.block_size
        self.super = {
            'time_stamp': get_time_stamp(), 'data_size': 0, 'index_size': self.block_size,
            'magic': MAGIC, 'version': VERSION, 'total_blocks': total_blocks,
            'rsvd_blocks': 1, 'block_size': BLOCK_SIZE_EXP, 'crc': 0
        }
        self.write_super()
        self.handle.seek(0); self.handle.write(b'\x00' * self.block_size)
        self.write_super() 
        idx_start = (total_blocks * self.block_size) - self.block_size
        self.handle.seek(idx_start); self.handle.write(b'\x00' * self.block_size)
        
        vol_id = bytearray(ENTRY_SIZE); vol_id[0] = TYPE_VOL_ID
        struct.pack_into("<xHq", vol_id, 0, 0, get_time_stamp())
        vol_id[12:12+10] = b"SFS_VOLUME"; vol_id[1] = calc_crc(vol_id)
        
        start_m = bytearray(ENTRY_SIZE); start_m[0] = TYPE_START; start_m[1] = calc_crc(start_m)
        self.handle.seek(idx_start); self.handle.write(start_m)
        self.handle.seek((total_blocks * self.block_size) - 64); self.handle.write(vol_id)
        
        mid = (self.block_size - 128) // 64
        if mid > 0:
            unused = bytearray(ENTRY_SIZE); unused[0] = TYPE_UNUSED; unused[1] = calc_crc(unused)
            self.handle.seek(idx_start + 64)
            for _ in range(mid): self.handle.write(unused)

    def read_super(self):
        self.handle.seek(SUPER_OFFSET)
        data = self.handle.read(SUPER_SIZE)
        if len(data) < SUPER_SIZE: raise Exception("File too small for SuperBlock")
        unpacked = struct.unpack(FMT_SUPER, data)
        if not validate_crc(data): print("Warning: SuperBlock CRC mismatch.")
        self.super = {
            'time_stamp': unpacked[0], 'data_size': unpacked[1], 'index_size': unpacked[2],
            'magic': unpacked[3], 'version': unpacked[4], 'total_blocks': unpacked[5],
            'rsvd_blocks': unpacked[6], 'block_size': unpacked[7], 'crc': unpacked[8]
        }
        if self.super['magic'] != MAGIC: raise Exception("Invalid Magic Signature")

    def write_super(self):
        temp = struct.pack(FMT_SUPER, self.super['time_stamp'], self.super['data_size'], self.super['index_size'],
            self.super['magic'], self.super['version'], self.super['total_blocks'],
            self.super['rsvd_blocks'], self.super['block_size'], 0)
        self.super['crc'] = calc_crc(temp)
        final = struct.pack(FMT_SUPER, self.super['time_stamp'], self.super['data_size'], self.super['index_size'],
            self.super['magic'], self.super['version'], self.super['total_blocks'],
            self.super['rsvd_blocks'], self.super['block_size'], self.super['crc'])
        self.handle.seek(SUPER_OFFSET); self.handle.write(final)

    def read_index(self):
        idx_size = self.super['index_size']
        start_offset = (self.super['total_blocks'] * self.block_size) - idx_size
        self.handle.seek(start_offset)
        raw_index = self.handle.read(idx_size)
        self.entries = []
        count = len(raw_index) // 64
        i = 0
        while i < count:
            chunk = raw_index[i*64 : (i+1)*64]
            entry = SFSEntry(chunk)
            if not entry.is_valid: 
                i+=1; continue
            if entry.entry_type == TYPE_UNUSED:
                i += 1; continue
            for c in range(entry.num_cont):
                i += 1
                if i < count: entry.continuations.append(raw_index[i*64 : (i+1)*64])
            self.entries.append(entry); i += 1

    def find_file(self, path):
        self.read_index()
        for e in self.entries:
            if e.entry_type == TYPE_FILE and e.get_name() == path: return e
        return None

    def read_file_content(self, path):
        entry = self.find_file(path)
        if not entry: raise FileNotFoundError("File not found in SFS image")
        start = struct.unpack_from("<Q", entry.raw, 11)[0]
        length = struct.unpack_from("<Q", entry.raw, 27)[0]
        self.handle.seek(start * self.block_size)
        return self.handle.read(length)

    def import_file(self, host_path, sfs_path):
        parts = sfs_path.split('/')
        if len(parts) > 1:
            cum_path = ""
            for p in parts[:-1]:
                cum_path = (cum_path + "/" + p) if cum_path else p
                exists = False
                self.read_index() 
                for e in self.entries:
                    if e.entry_type == TYPE_DIR and e.get_name() == cum_path:
                        exists = True; break
                if not exists:
                    def pack_dir(buf): struct.pack_into("<q", buf, 3, get_time_stamp())
                    self.write_entry(TYPE_DIR, cum_path, pack_dir)
        with open(host_path, 'rb') as f: content = f.read()
        start, end, length = self.add_file_content(content)
        if length == 0: start, end = 0, 0
        def pack_file(buf): struct.pack_into("<qQQQ", buf, 3, get_time_stamp(), start, end, length)
        self.write_entry(TYPE_FILE, sfs_path, pack_file)

    def add_file_content(self, content: bytes) -> Tuple[int, int, int]:
        current = self.super['data_size']; start = self.super['rsvd_blocks'] + current
        flen = len(content)
        if flen == 0: return (0, 0, 0)
        needed = math.ceil(flen / self.block_size)
        end = start + needed - 1
        idx_start = self.super['total_blocks'] - math.ceil(self.super['index_size'] / self.block_size)
        if end >= idx_start: raise Exception("Not enough free space in volume. Resize volume.")
        self.handle.seek(start * self.block_size); self.handle.write(content)
        pad = (needed * self.block_size) - flen
        if pad > 0: self.handle.write(b'\x00' * pad)
        self.super['data_size'] += needed; self.super['time_stamp'] = get_time_stamp()
        self.write_super()
        return (start, end, flen)

    def write_entry(self, entry_type, name, meta_data_packer):
        name_encoded = name.encode('utf-8')
        max_first = FILE_NAME_LEN if entry_type == TYPE_FILE else DIR_NAME_LEN
        continuations = []
        if len(name_encoded) > max_first:
            rem = name_encoded[max_first:]
            while len(rem) > 0:
                chunk = rem[:64]; c = bytearray(64); c[:len(chunk)] = chunk; continuations.append(c); rem = rem[64:]
            if len(name_encoded) > max_first and (len(name_encoded) - max_first) % 64 == 0:
                 continuations.append(bytearray(64))
        else:
            if len(name_encoded) == max_first: continuations.append(bytearray(64))
        
        num_cont = len(continuations)
        entry = bytearray(ENTRY_SIZE); entry[0] = entry_type; entry[2] = num_cont
        meta_data_packer(entry)
        part = name_encoded[:max_first]
        if entry_type == TYPE_FILE: entry[35:35+len(part)] = part
        elif entry_type == TYPE_DIR: entry[11:11+len(part)] = part
        entry[1] = 0; total_data = entry + b"".join(continuations); entry[1] = calc_crc(total_data)
        
        idx_size = self.super['index_size']
        idx_base = (self.super['total_blocks'] * self.block_size) - idx_size
        self.handle.seek(idx_base); raw = self.handle.read(idx_size)
        
        needed = 1 + num_cont; found_idx = -1; slots = len(raw) // 64
        start_search = 1 if raw[0] == TYPE_START else 0
        for i in range(start_search, slots - needed + 1):
            is_free = True
            for k in range(needed):
                if raw[(i+k)*64] != TYPE_UNUSED: is_free = False; break
            if is_free: found_idx = i; break
                
        if found_idx == -1:
            idx_blocks = idx_size // self.block_size
            start_block = self.super['total_blocks'] - idx_blocks
            if start_block - 1 <= self.super['rsvd_blocks'] + self.super['data_size']:
                 raise Exception("Disk full (Index cannot expand). Resize volume.")
            self.handle.seek(idx_base); old_index = bytearray(self.handle.read(idx_size))
            self.super['index_size'] += self.block_size
            new_idx_base = idx_base - self.block_size
            new_part = bytearray(self.block_size)
            for z in range(0, self.block_size, 64): new_part[z] = TYPE_UNUSED
            if old_index[0] == TYPE_START:
                new_part[0:64] = old_index[0:64]; old_index[0] = TYPE_UNUSED; old_index[1] = calc_crc(old_index[0:64])
            self.handle.seek(new_idx_base); self.handle.write(new_part + old_index)
            self.write_super()
            return self.write_entry(entry_type, name, meta_data_packer)
            
        offset = idx_base + (found_idx * 64)
        self.handle.seek(offset); self.handle.write(entry)
        for c in continuations: self.handle.write(c)

    def resize_volume(self, new_size_mb):
        new_bytes = int(new_size_mb * 1024 * 1024)
        old_blocks = self.super['total_blocks']
        if new_bytes <= old_blocks * self.block_size:
            print("Size unchanged or too small. Use 'shrink' to reduce.")
            return
        idx_size = self.super['index_size']
        idx_start = (old_blocks * self.block_size) - idx_size
        self.handle.seek(idx_start); index_data = self.handle.read(idx_size)
        self.handle.seek(idx_start); self.handle.write(b'\x00' * idx_size)
        self.handle.truncate(new_bytes)
        new_total = new_bytes // self.block_size
        new_start = (new_total * self.block_size) - idx_size
        self.handle.seek(new_start); self.handle.write(index_data)
        self.super['total_blocks'] = new_total
        self.write_super()
        print(f"Resized to {new_size_mb} MB.")

    def shrink_to_fit(self):
        rsvd = self.super['rsvd_blocks']; data = self.super['data_size']
        idx = math.ceil(self.super['index_size'] / self.block_size)
        old_total = self.super['total_blocks']
        new_total = rsvd + data + idx
        if new_total >= old_total:
            print("Volume is already at minimum size.")
            return
        print(f"Shrinking to {new_total} blocks...")
        idx_size = self.super['index_size']
        old_start = (old_total * self.block_size) - idx_size
        self.handle.seek(old_start); index_data = self.handle.read(idx_size)
        new_start = (rsvd + data) * self.block_size
        self.handle.seek(new_start); self.handle.write(index_data)
        self.handle.truncate(new_total * self.block_size)
        self.super['total_blocks'] = new_total
        self.write_super()
        print("Shrink complete.")

    def defrag(self):
        self.read_index() 
        files = []
        others = []
        vol_id_entry = None
        for e in self.entries:
            if e.entry_type == TYPE_FILE:
                start = struct.unpack_from("<Q", e.raw, 11)[0]
                files.append((start, e))
            elif e.entry_type == TYPE_VOL_ID: vol_id_entry = e
            else: others.append(e)

        if vol_id_entry is None:
            print("Warning: Volume ID entry missing. Creating a new one.")
            vol_id_entry = SFSEntry(); vol_id_entry.raw[0] = TYPE_VOL_ID
            struct.pack_into("<xHq", vol_id_entry.raw, 0, 0, get_time_stamp())
            vol_id_entry.raw[12:12+10] = b"SFS_VOLUME"; vol_id_entry.update_crc()

        files.sort(key=lambda x: x[0])
        current = self.super['rsvd_blocks']
        print(f"Defragmenting {len(files)} files...")
        
        for old_start, entry in files:
            flen = struct.unpack_from("<Q", entry.raw, 27)[0]
            needed = math.ceil(flen / self.block_size) if flen > 0 else 0
            new_start = current
            new_end = new_start + needed - 1 if needed > 0 else 0
            
            if old_start != new_start and needed > 0:
                self.handle.seek(old_start * self.block_size); data = self.handle.read(flen) 
                self.handle.seek(new_start * self.block_size); self.handle.write(data)
                pad = (needed * self.block_size) - flen
                if pad > 0: self.handle.write(b'\x00' * pad)
            
            struct.pack_into("<QQ", entry.raw, 11, new_start, new_end)
            entry.update_crc()
            current += needed
            
        self.super['data_size'] = current - self.super['rsvd_blocks']
        idx_size = self.super['index_size']
        idx_start = (self.super['total_blocks'] * self.block_size) - idx_size
        self.handle.seek(idx_start)
        
        written = 0
        all_e = others + [f[1] for f in files]
        for e in all_e:
            self.handle.write(e.raw); written += 64
            for c in e.continuations: self.handle.write(c); written += 64
                
        rem = idx_size - written - 64 
        if rem < 0: print(f"Warning: Index overflow by {abs(rem)} bytes.")
        else:
            unused = bytearray(ENTRY_SIZE); unused[0] = TYPE_UNUSED; unused[1] = calc_crc(unused)
            for _ in range(rem // 64): self.handle.write(unused)
                
        self.handle.seek(idx_start + idx_size - 64)
        self.handle.write(vol_id_entry.raw)
        self.write_super()
        print("Defragmentation complete.")

# --- CLI Handlers ---

def cmd_create(args):
    total_data_size = 0; file_count = 0; dir_count = 0; ignore_list = set()
    if args.ignore:
        for i in args.ignore:
            name = os.path.basename(os.path.normpath(i))
            if name: ignore_list.add(name)
    
    for root, dirs, files in os.walk(args.folder):
        dirs[:] = [d for d in dirs if d not in ignore_list]
        dir_count += len(dirs)
        for file in files:
            fp = os.path.join(root, file); total_data_size += os.path.getsize(fp); file_count += 1
            
    data_blks = 0
    for root, dirs, files in os.walk(args.folder):
        dirs[:] = [d for d in dirs if d not in ignore_list]
        for file in files:
            fp = os.path.join(root, file); sz = os.path.getsize(fp)
            if sz > 0: data_blks += math.ceil(sz / 512)

    est_entries = 2 + (file_count * 2) + (dir_count * 2)
    idx_blks = math.ceil((est_entries * 64) / 512)
    total = 1 + data_blks + idx_blks
    req = max(65536, total * 512)
    final = 1 << (req - 1).bit_length()
    
    print(f"Scanned {file_count} files. Data: {total_data_size} bytes.")
    print(f"Estimated blocks: {total}. Volume Size: {final/1024/1024:.2f} MB")
    
    vol = SFSVolume(args.image, 'wb+', init_size=final)
    
    curr = 0
    for root, dirs, files in os.walk(args.folder):
        dirs[:] = [d for d in dirs if d not in ignore_list]
        for file in files:
            curr += 1
            fp = os.path.join(root, file)
            rp = os.path.relpath(fp, args.folder).replace("\\", "/")
            vol.import_file(fp, rp)
            if args.verbose: print(f"Added: {rp}")
            if file_count > 0 and not args.verbose:
                p = int((curr / file_count) * 100)
                sys.stdout.write(f"\rProgress: {p}%"); sys.stdout.flush()
    if not args.verbose: print()

def cmd_add(args):
    vol = SFSVolume(args.image, 'rb+')
    vol.import_file(args.file, os.path.basename(args.file))
    print("File added.")

def cmd_list(args):
    vol = SFSVolume(args.image, 'rb')
    vol.read_index()
    print(f"{'Type':<6} {'Size':<10} {'Name'}")
    print("-" * 60)
    for e in vol.entries:
        if e.entry_type == TYPE_FILE:
            l = struct.unpack_from("<Q", e.raw, 27)[0]
            print(f"{'FILE':<6} {l:<10} {e.get_name()}")
        elif e.entry_type == TYPE_DIR: print(f"{'DIR':<6} {'-':<10} {e.get_name()}")
        elif e.entry_type == TYPE_VOL_ID: print(f"VOLID: {e.get_name()}")

def cmd_cat(args):
    vol = SFSVolume(args.image, 'rb')
    try: sys.stdout.buffer.write(vol.read_file_content(args.path))
    except Exception as e: print(e, file=sys.stderr)

def cmd_resize(args): SFSVolume(args.image, 'rb+').resize_volume(float(args.size))
def cmd_shrink(args): SFSVolume(args.image, 'rb+').shrink_to_fit()
def cmd_defrag(args): SFSVolume(args.image, 'rb+').defrag()

def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest='command', required=True)
    
    p = subparsers.add_parser('create'); p.add_argument('image'); p.add_argument('folder')
    p.add_argument('-v', '--verbose', action='store_true'); p.add_argument('-ignore', action='append')
    p.set_defaults(func=cmd_create)
    
    p = subparsers.add_parser('add'); p.add_argument('image'); p.add_argument('file')
    p.add_argument('-v', '--verbose', action='store_true'); p.set_defaults(func=cmd_add)
    
    p = subparsers.add_parser('list'); p.add_argument('image'); p.set_defaults(func=cmd_list)
    p = subparsers.add_parser('cat'); p.add_argument('image'); p.add_argument('path'); p.set_defaults(func=cmd_cat)
    p = subparsers.add_parser('resize'); p.add_argument('image'); p.add_argument('size'); p.set_defaults(func=cmd_resize)
    p = subparsers.add_parser('shrink'); p.add_argument('image'); p.set_defaults(func=cmd_shrink)
    p = subparsers.add_parser('defrag'); p.add_argument('image'); p.set_defaults(func=cmd_defrag)
    
    args = parser.parse_args(); args.func(args)

if __name__ == '__main__': main()
