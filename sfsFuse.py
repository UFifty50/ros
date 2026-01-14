#!/usr/bin/env python3
import os, sys, errno, time, struct, stat, math, traceback
try: from fuse import FUSE, FuseOSError, Operations
except ImportError: sys.exit("Install fusepy")

ENTRY_SIZE = 64; SUPER_OFFSET = 0x18E; SUPER_SIZE = 42; 
TYPE_VOL_ID=1; TYPE_START=2; TYPE_UNUSED=16; TYPE_DIR=17; TYPE_FILE=18
TYPE_DIR_DEL=25; TYPE_FILE_DEL=26
FMT_SUPER = "<qQQ3sBQIBB"

def get_time_stamp(): return int(time.time() * 65536)
def calc_crc(data): return (256 - (sum(data) & 0xFF)) & 0xFF

class SFSEntry:
    def __init__(self, raw, offset=0):
        self.raw = bytearray(raw); self.entry_type = self.raw[0]; self.index_offset = offset
        self.continuations = []
    @property
    def num_cont(self): return self.raw[2] if self.entry_type in [17,18,25,26] else 0
    def get_name(self):
        nb = self.raw[35:35+29] if self.entry_type in [18,26] else self.raw[11:11+53]
        if self.entry_type == 1: nb = self.raw[12:12+52]
        for c in self.continuations: nb += c
        try: return nb.split(b'\x00')[0].decode('utf-8')
        except: return "<Invalid>"
    def get_attr(self):
        ts = struct.unpack_from("<q", self.raw, 3)[0] / 65536.0
        if self.entry_type == TYPE_DIR: return {'st_mode':(stat.S_IFDIR|0o777),'st_nlink':2,'st_size':0,'st_mtime':ts}
        fl = struct.unpack_from("<Q", self.raw, 27)[0]
        return {'st_mode':(stat.S_IFREG|0o666),'st_nlink':1,'st_size':fl,'st_mtime':ts}

class SFSVolume:
    def __init__(self, path):
        self.handle = open(path, 'rb+'); self.block_size = 512; self.write_buffers = {}; self.fd=10
        self._read_super(); self._read_index(); self._build_tree()
    
    def _read_super(self):
        self.handle.seek(SUPER_OFFSET); data = self.handle.read(SUPER_SIZE)
        u = struct.unpack(FMT_SUPER, data)
        self.super = {'time':u[0],'data':u[1],'idx':u[2],'magic':u[3],'ver':u[4],'total':u[5],'rsvd':u[6],'blk_exp':u[7]}
        self.block_size = 1 << (self.super['blk_exp'] + 7)

    def _write_super(self):
        self.super['time'] = get_time_stamp()
        t = struct.pack(FMT_SUPER, self.super['time'], self.super['data'], self.super['idx'],
            self.super['magic'], self.super['ver'], self.super['total'], self.super['rsvd'], self.super['blk_exp'], 0)
        crc = calc_crc(t)
        f = struct.pack(FMT_SUPER, self.super['time'], self.super['data'], self.super['idx'],
            self.super['magic'], self.super['ver'], self.super['total'], self.super['rsvd'], self.super['blk_exp'], crc)
        self.handle.seek(SUPER_OFFSET); self.handle.write(f)

    def _read_index(self):
        isz = self.super['idx']; start = (self.super['total']*self.block_size)-isz
        self.handle.seek(start); raw = self.handle.read(isz)
        self.entries = []; cnt = len(raw)//64; i=0
        while i < cnt:
            off = i*64; e = SFSEntry(raw[off:off+64], start+off)
            if e.entry_type == TYPE_UNUSED: self.entries.append(e); i+=1; continue
            for _ in range(e.num_cont):
                i+=1; 
                if i<cnt: e.continuations.append(raw[i*64:(i+1)*64])
            self.entries.append(e); i+=1

    def _build_tree(self):
        self.path_map = {}; self.dir_children = {'':[]}
        for e in self.entries:
            if e.entry_type in [17,18]:
                p = e.get_name().replace('\\','/'); self.path_map[p] = e
                par, nm = (p.rsplit('/',1) if '/' in p else ("",p))
                self.dir_children.setdefault(par,[]).append(nm)

    def count_free_blocks(self):
        used = 0
        for e in self.entries:
            if e.entry_type == TYPE_FILE:
                s = struct.unpack_from("<Q", e.raw, 11)[0]
                en = struct.unpack_from("<Q", e.raw, 19)[0]
                if en >= s and s > 0: used += (en-s+1)
        rsvd = self.super['rsvd']; idx = math.ceil(self.super['idx']/self.block_size)
        return self.super['total'] - rsvd - idx - used

    def _allocate_blocks(self, count):
        if count == 0: return 0
        ints = []
        for e in self.entries:
            if e.entry_type == TYPE_FILE:
                s = struct.unpack_from("<Q", e.raw, 11)[0]
                en = struct.unpack_from("<Q", e.raw, 19)[0]
                if s>0: ints.append((s,en))
        ints.sort()
        merged = []
        if ints:
            cs, ce = ints[0]
            for ns, ne in ints[1:]:
                if ns <= ce+1: ce = max(ce, ne)
                else: merged.append((cs,ce)); cs,ce = ns,ne
            merged.append((cs,ce))
        
        ss = self.super['rsvd']
        for s,e in merged:
            if s - ss >= count: return ss
            ss = e + 1
        
        idx_start = self.super['total'] - math.ceil(self.super['idx']/self.block_size)
        if (idx_start - ss) >= count:
            ne = ss + count
            if ne > self.super['rsvd'] + self.super['data']:
                self.super['data'] = ne - self.super['rsvd']; self._write_super()
            return ss
        raise FuseOSError(errno.ENOSPC)

    def _write_entry(self, type, name, packer):
        ne = name.encode('utf-8'); conts = []
        mf = 29 if type==18 else 53
        rem = ne[mf:] if len(ne)>mf else b''
        if len(ne)>=mf:
            while rem:
                c = bytearray(64); c[:len(rem[:64])] = rem[:64]; conts.append(c); rem = rem[64:]
            if len(ne) >= mf + len(conts)*64: conts.append(bytearray(64))
        
        need = 1+len(conts); start_idx=-1; seq=0
        for i,e in enumerate(self.entries):
            seq = seq+1 if e.entry_type in [16,25,26] else 0
            if seq == need: start_idx = i-need+1; break
        if start_idx == -1: raise FuseOSError(errno.ENOSPC)

        m = bytearray(64); m[0]=type; m[2]=len(conts); packer(m)
        p = ne[:mf]; m[11 if type==17 else 35 : (11 if type==17 else 35)+len(p)] = p
        m[1] = 0; m[1] = calc_crc(m + b"".join(conts))
        
        base = self.entries[start_idx].index_offset
        self.handle.seek(base); self.handle.write(m)
        for c in conts: self.handle.write(c)
        self._read_index(); self._build_tree()

    def create(self, path):
        p = path.lstrip('/').replace('\\','/')
        self._write_entry(18, p, lambda b: struct.pack_into("<qQQQ",b,3,get_time_stamp(),0,0,0))
        self.fd+=1; return self.fd

    def write_buf(self, path, buf, off):
        p = path.lstrip('/').replace('\\','/')
        if p not in self.write_buffers:
            if p in self.path_map:
                e = self.path_map[p]
                sz = struct.unpack_from("<Q", e.raw, 27)[0]
                s = struct.unpack_from("<Q", e.raw, 11)[0]
                if sz>0: self.handle.seek(s*self.block_size); self.write_buffers[p] = bytearray(self.handle.read(sz))
                else: self.write_buffers[p] = bytearray()
            else: self.write_buffers[p] = bytearray()
        
        b = self.write_buffers[p]
        end = off + len(buf)
        
        # PRE-CHECK SPACE
        req_blocks = math.ceil(end / self.block_size)
        cur_blocks = 0
        if p in self.path_map:
             e = self.path_map[p]
             s = struct.unpack_from("<Q", e.raw, 11)[0]
             en = struct.unpack_from("<Q", e.raw, 19)[0]
             if s > 0: cur_blocks = en - s + 1
        
        cost = req_blocks - cur_blocks
        if cost > 0:
             free = self.count_free_blocks()
             if cost > free: raise FuseOSError(errno.ENOSPC)

        if end > len(b): b.extend(b'\0'*(end-len(b)))
        b[off:end] = buf
        return len(buf)

    def flush(self, path):
        p = path.lstrip('/').replace('\\','/')
        if p not in self.write_buffers: return
        c = self.write_buffers[p]; sz = len(c); blks = math.ceil(sz/self.block_size)
        try: sb = self._allocate_blocks(blks)
        except FuseOSError: 
            self.unlink(path); del self.write_buffers[p]; raise
        
        if sz>0:
            self.handle.seek(sb*self.block_size); self.handle.write(c)
            pad = blks*self.block_size - sz
            if pad>0: self.handle.write(b'\0'*pad)
        
        self.unlink(path)
        eb = sb+blks-1 if blks>0 else 0; 
        if blks==0: sb=0
        self._write_entry(18, p, lambda b: struct.pack_into("<qQQQ",b,3,get_time_stamp(),sb,eb,sz))
        del self.write_buffers[p]

    def unlink(self, path):
        p = path.lstrip('/').replace('\\','/')
        if p not in self.path_map: return
        e = self.path_map[p]; nt = 26 if e.entry_type==18 else 25
        self.handle.seek(e.index_offset); self.handle.write(bytes([nt]))
        self._read_index(); self._build_tree()

class SFSFuse(Operations):
    def __init__(self, f): self.v = SFSVolume(f)
    def __call__(self, o, *a):
        try: return getattr(self, o)(*a)
        except FuseOSError: raise
        except: traceback.print_exc(); raise FuseOSError(errno.EIO)
    def getattr(self, p, fh=None):
        pp=p.lstrip('/').replace('\\','/')
        if pp=='': return {'st_mode':0o40777,'st_nlink':2}
        if pp not in self.v.path_map: raise FuseOSError(errno.ENOENT)
        return self.v.path_map[pp].get_attr()
    def readdir(self, p, fh):
        pp=p.lstrip('/').replace('\\','/')
        yield '.'; yield '..'
        for c in self.v.dir_children.get(pp, []): yield c
    def read(self, p, l, o, fh):
        pp=p.lstrip('/').replace('\\','/')
        if pp not in self.v.path_map: raise FuseOSError(errno.ENOENT)
        e = self.v.path_map[pp]; s = struct.unpack_from("<Q", e.raw, 11)[0]
        self.v.handle.seek(s*self.v.block_size + o); return self.v.handle.read(l)
    def create(self, p, m, fi=None): return self.v.create(p)
    def open(self, p, f): return self.v.fd+1
    def write(self, p, b, o, fh): return self.v.write_buf(p, b, o)
    def release(self, p, fh): self.v.flush(p); return 0
    def unlink(self, p): self.v.unlink(p)
    def mkdir(self, p, m): self.v._write_entry(17, p.lstrip('/').replace('\\','/'), lambda b: struct.pack_into("<q",b,3,get_time_stamp()))
    def rmdir(self, p):
        pp=p.lstrip('/').replace('\\','/')
        if self.v.dir_children.get(pp): raise FuseOSError(errno.ENOTEMPTY)
        self.v.unlink(p)
    def statfs(self, p): 
        f = self.v.count_free_blocks()
        return {'f_bsize':512,'f_frsize':512,'f_blocks':self.v.super['total'],'f_bfree':f,'f_bavail':f}
    def chmod(self,p,m): return 0
    def chown(self,p,u,g): return 0
    def utimens(self,p,t=None): return 0
    def access(self,p,m): return 0
    def rename(self, o, n):
        op=o.lstrip('/').replace('\\','/'); np=n.lstrip('/').replace('\\','/')
        e = self.v.path_map[op]
        if e.entry_type == 17:
             self.v._write_entry(17, np, lambda b: struct.pack_into("<q",b,3,get_time_stamp()))
        else:
             s = struct.unpack_from("<Q",e.raw,11)[0]; en = struct.unpack_from("<Q",e.raw,19)[0]; sz=struct.unpack_from("<Q",e.raw,27)[0]
             self.v._write_entry(18, np, lambda b: struct.pack_into("<qQQQ",b,3,get_time_stamp(),s,en,sz))
        self.v.unlink(o)

def main():
    if len(sys.argv)!=3: sys.exit("Usage: python sfsFuse.py <img> <mnt>")
    mnt = sys.argv[2]
    if os.name!='nt' and not os.path.exists(mnt): os.makedirs(mnt)
    FUSE(SFSFuse(sys.argv[1]), mnt, nothreads=True, foreground=True)

if __name__=='__main__': main()
