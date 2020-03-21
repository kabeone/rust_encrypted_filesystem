use crate::get_fs::FileInfo;
use crate::path::get_entry_path;
use crate::file::{get_new_file_handle, FILE_HANDLES};

use fuse::{Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory, FileAttr, ReplyWrite, ReplyOpen, ReplyEmpty};
use time::Timespec;
use libc::ENOENT;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use crate::setattr::change_file_attr;
use std::fs::OpenOptions;
use std::io::Write;

pub static mut PATH_MOUNTPOINT : Option<PathBuf> = None;
pub static mut PATH_SRC : Option<PathBuf> = None;
pub static mut FILE_ENTRIES : Vec<FileInfo> = Vec::new();

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

pub struct FS;

impl Filesystem for FS {

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        unsafe {
            for entry in FILE_ENTRIES.iter() {

                if entry.parent_inode == parent && entry.name.to_str() == name.to_str() {
                    reply.entry(&TTL, &entry.attribute, 0);
                    return;
                }

            }
        }
        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        unsafe {
            for entry in FILE_ENTRIES.iter() {

                if entry.attribute.ino == ino {
                    return reply.attr(&TTL, &entry.attribute);
                }

            }
        }
        return reply.error(ENOENT);
    }

    fn setattr(&mut self, _req: &Request, ino: u64, _mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>,
        atime: Option<Timespec>, mtime: Option<Timespec>, fh: Option<u64>, crtime: Option<Timespec>, chgtime: Option<Timespec>,
        bkuptime: Option<Timespec>, flags: Option<u32>, reply: ReplyAttr) {

        let n = change_file_attr(ino, _mode, uid, gid, size, atime, mtime, fh, crtime, chgtime, bkuptime, flags);

        match n {
            -1 => reply.error(ENOENT),
            n => {
                unsafe {
                    reply.attr(&TTL, &FILE_ENTRIES.get(n as usize).unwrap().attribute)
                }
            }
        }
    }

    fn mknod(&mut self, _req: &Request, parent: u64, name: &OsStr, mode: u32, rdev: u32, reply: ReplyEntry) {
        let mut path;
        let inode;

        unsafe {
            crate::get_fs::A += 1;
            inode = crate::get_fs::A;
            path = String::from(PATH_SRC.clone().unwrap().to_str().unwrap());
            get_entry_path(&mut path, parent, &OsString::from(name));
        }

        match std::fs::File::create(&path) {
            Ok(_t) => (),
            Err(e) => {
                println!("Error: {}", e);
                return reply.error(ENOENT);
            },
        }

        let entry :FileInfo = FileInfo {
            attribute : FileAttr {
                ino: inode,
                size: 0,
                blocks: 1,
                atime: crate::get_fs::CREATE_TIME,
                mtime: crate::get_fs::CREATE_TIME,
                ctime: crate::get_fs::CREATE_TIME,
                crtime: crate::get_fs::CREATE_TIME,
                kind: fuse::FileType::RegularFile,
                perm: mode as u16,
                nlink: 0,
                uid: 501,
                gid: 20,
                rdev,
                flags: 0,
            },
            parent_inode : parent,
            path : OsString::from(path),
            name : name.to_os_string()
        };

        unsafe {
            FILE_ENTRIES.push(entry);
            return reply.entry(&TTL, &FILE_ENTRIES.last().unwrap().attribute, 0);
        }
    }

    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, mode: u32, reply: ReplyEntry) {
        let mut path;
        let inode;

        unsafe {
            crate::get_fs::A += 1;
            inode = crate::get_fs::A;
            path = String::from(PATH_SRC.clone().unwrap().to_str().unwrap());
            get_entry_path(&mut path, parent, &OsString::from(name));
        }

        match std::fs::create_dir(&path) {
            Ok(_t) => (),
            Err(e) => {
                println!("Error: {}", e);
                return reply.error(ENOENT);
            },
        }

        let entry :FileInfo = FileInfo {
            attribute : FileAttr {
                ino: inode,
                size: 64,
                blocks: 2,
                atime: crate::get_fs::CREATE_TIME,
                mtime: crate::get_fs::CREATE_TIME,
                ctime: crate::get_fs::CREATE_TIME,
                crtime: crate::get_fs::CREATE_TIME,
                kind: fuse::FileType::Directory,
                perm: mode as u16,
                nlink: 0,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
            },
            parent_inode : parent,
            path : OsString::from(path),
            name : name.to_os_string()
        };

        unsafe {
            FILE_ENTRIES.push(entry);
            return reply.entry(&TTL, &FILE_ENTRIES.last().unwrap().attribute, 0);
        }
    }

    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        let handle = get_new_file_handle(ino);
        if handle == 0 {
            reply.error(ENOENT);
        } else {
            reply.opened(handle, flags);
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {
        unsafe {
            for entry in FILE_ENTRIES.iter() {
                if entry.attribute.ino == ino {
                    let content = match std::fs::read_to_string(&entry.path) {
                        Ok(t) => t,
                        Err(e) => {
                            println!("Error: read: {}", e);
                            reply.error(ENOENT);
                            return;
                        }
                    };
                    let mut decode = content.as_bytes().to_owned();

                    let mut i = 0;
                    while i < decode.len() {
                        if decode[i] == 0 {
                            decode[i] = 255;
                        } else {
                            decode[i] -= 1;
                        }
                        i += 1;
                    }

                    reply.data(&decode[offset as usize..]);
                    return;
                }
            }
        }
        reply.error(ENOENT);
        return;
    }

    fn write(&mut self, _req: &Request, _ino: u64, fh: u64, _offset: i64, data: &[u8], _flags: u32, reply: ReplyWrite) {
        let mut a = 0;
        let mut encode = data.clone().to_owned();
        let mut i = 0;

        while i < encode.len() {
            if encode[i] == 255 {
                encode[i] = 0;
            } else {
                encode[i] += 1;
            }
            i += 1;
        }

        unsafe {
            for handle in FILE_HANDLES.iter() {
                if handle.handle == fh {
                    a += 1;
                    let mut file = match OpenOptions::new().write(true).truncate(true).open(handle.path.clone()) {
                        Ok(t) => t,
                        Err(e) => {
                            println!("Error: open: {}", e);
                            return reply.error(ENOENT);
                        }
                    };
                    match file.write(&encode) {
                        Ok(_t) => (),
                        Err(e) => println!("Error: write: {}", e)
                    }
                }
            }
        }

        match a {
            0 => reply.error(ENOENT),
            _ => reply.written(encode.len() as u32)
        }
    }

    fn flush(&mut self, _req: &Request, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        reply.ok();
    }

    fn release(&mut self, _req: &Request, _ino: u64, fh: u64, _flags: u32, _lock_owner: u64, _flush: bool, reply: ReplyEmpty) {
        let mut i = 0;
        let mut a = 0;

        unsafe {
            for n in crate::file::FILE_HANDLES.iter() {
                if n.handle == fh {
                    a = 1;
                    break;
                }
                i += 1;
            }
            if a == 1 {
                crate::file::FILE_HANDLES.remove(i);
            }
        }
        reply.ok();
    }

    fn fsync(&mut self, _req: &Request, _ino: u64, _fh: u64, _datasync: bool, reply: ReplyEmpty) {
        reply.ok();
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        let mut a = 0;
        let mut entries = vec![];

        unsafe {
            for entry in FILE_ENTRIES.iter() {
                if entry.parent_inode == ino {
                    entries.push((entry.attribute.ino, entry.attribute.kind, entry.name.clone()));
                    a += 1;
                }
            }
        }
        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        if a == 0 {
            reply.error(ENOENT);
            return;
        }
        reply.ok();
        return;
    }
}