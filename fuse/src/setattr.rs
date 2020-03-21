use crate::fs::FILE_ENTRIES;
use crate::get_fs::FileInfo;
use time::Timespec;
use fuse::FileAttr;

fn set_default(inode :u64) -> Option<FileInfo> {
    let infos;

    unsafe {
        for entry in FILE_ENTRIES.iter() {
            if entry.attribute.ino == inode {
                infos = FileInfo {
                    attribute: FileAttr {
                        ino: entry.attribute.ino,
                        size: entry.attribute.size,
                        blocks: entry.attribute.blocks,
                        atime: entry.attribute.atime,
                        mtime: entry.attribute.mtime,
                        ctime: entry.attribute.ctime,
                        crtime: entry.attribute.crtime,
                        kind: entry.attribute.kind,
                        perm: entry.attribute.perm,
                        nlink: entry.attribute.nlink,
                        uid: entry.attribute.uid,
                        gid: entry.attribute.gid,
                        rdev: entry.attribute.rdev,
                        flags: entry.attribute.flags,
                    },
                    parent_inode: entry.parent_inode,
                    name: entry.name.clone(),
                    path: entry.path.clone()
                };
                return Some(infos);
            }
        }
    }
    None
}

pub fn change_file_attr(inode: u64, mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>,
                        atime: Option<Timespec>, mtime: Option<Timespec>, _fh: Option<u64>, crtime: Option<Timespec>,
                        chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, flags: Option<u32>) -> i32{


    let mut new = match set_default(inode) {
        Some(t) => t,
        None => return -1
    };

    match mode {
        Some(t) => new.attribute.perm = t as u16,
        None => (),
    }
    match uid {
        Some(t) => new.attribute.uid = t,
        None => (),
    }
    match gid {
        Some(t) => new.attribute.gid = t,
        None => (),
    }
    match size {
        Some(t) => new.attribute.size = t,
        None => (),
    }
    match atime {
        Some(t) => new.attribute.atime = t,
        None => (),
    }
    match mtime {
        Some(t) => new.attribute.mtime = t,
        None => (),
    }
    match crtime {
        Some(t) => new.attribute.crtime = t,
        None => (),
    }
    match chgtime {
        Some(t) => new.attribute.ctime = t,
        None => (),
    }
    match flags {
        Some(t) => new.attribute.flags = t,
        None => (),
    }

    unsafe {
        let mut i = 0;
        for entry in FILE_ENTRIES.iter() {
            if entry.attribute.ino == inode {
                break;
            }
            i += 1;
        }
        if i < FILE_ENTRIES.len() {
            FILE_ENTRIES[i] = new;
            return i as i32;
        }
    }
    -1
}