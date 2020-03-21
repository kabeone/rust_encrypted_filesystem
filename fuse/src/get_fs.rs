use std::env;
use std::path;
use std::fs;
use std::process::exit;
use fuse::FileAttr;
use std::fs::{ReadDir, read_dir, DirEntry};
use time::Timespec;
use std::os::macos::fs::MetadataExt;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub static mut A :u64 = 1;

pub const CREATE_TIME: Timespec = Timespec { sec: 0, nsec: 0 };

pub struct FileInfo {
    pub attribute : FileAttr,
    pub parent_inode : u64,
    pub name : OsString,
    pub path : OsString
}

fn add_info_entry(vec :&mut Vec<FileInfo>, entry :&DirEntry, parent_inode :u64) {
    let f_type :fuse::FileType;
    let inode;

    unsafe {
        A += 1;
        inode = A;
    }

    if entry.metadata().unwrap().file_type().is_symlink() {
        f_type = fuse::FileType::Symlink;
    } else if entry.metadata().unwrap().file_type().is_dir() {
        f_type = fuse::FileType::Directory;
    } else {
        f_type = fuse::FileType::RegularFile;
    }

    let infos :FileInfo = FileInfo {
        attribute: FileAttr {
            ino: inode,
            size: entry.metadata().unwrap().st_size(),
            blocks: 1,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: f_type,
            perm: 0o755,
            nlink: 0,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
        },
        parent_inode,
        name: entry.file_name(),
        path: entry.path().as_os_str().to_owned()
    };
    vec.push(infos);
}

fn set_root_dir(vec :&mut Vec<FileInfo>, path :& Path) {
    let infos :FileInfo = FileInfo {
        attribute: FileAttr {
            ino: 1,
            size: path.metadata().unwrap().st_size(),
            blocks: 1,
            atime: CREATE_TIME,
            mtime: CREATE_TIME,
            ctime: CREATE_TIME,
            crtime: CREATE_TIME,
            kind: fuse::FileType::Directory,
            perm: 0o777,
            nlink: 0,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
        },
        parent_inode: 0,
        name: match path.file_name() {
            Some(n) => n.to_owned(),
            None => {
                println!("Error cant mount filesystem: no name found for mountpoint root directory\n\t\tAre you trying to mount \"/\"?\
                \n\t\tElse please specify full path");
                exit(84);
            },
        },
        path : path.as_os_str().to_owned()
    };
    vec.push(infos);
}

fn get_fs_loop(dir_entries :ReadDir, parent_ino :u64) {

    unsafe {
        for entry in dir_entries {
            let entry = entry.unwrap();
            add_info_entry(&mut crate::fs::FILE_ENTRIES, &entry, parent_ino);

            if entry.metadata().unwrap().is_dir() {
                let dir = match read_dir(entry.path()) {
                    Ok(t) => t,
                    Err(e) => {
                        println!("Error: {}", e);
                        exit(84);
                    }
                };
                get_fs_loop(dir, A);
            }
        }
    }
}

pub fn get_fs() {
    let path = env::args_os().nth(1).unwrap();
    let path2 = env::args_os().nth(2).unwrap();

    unsafe {
        set_root_dir(&mut crate::fs::FILE_ENTRIES, path::Path::new(path.to_str().unwrap()));
        let dir_entry = match fs::read_dir(path::Path::new(&path.to_str().unwrap())) {
            Ok(t) => t,
            Err(e) => {
                println!("Error: {}", e);
                exit(84);
            }
        };

        get_fs_loop(dir_entry, 1);

        crate::fs::PATH_SRC = Some(PathBuf::from(path).canonicalize().unwrap());
        crate::fs::PATH_MOUNTPOINT = Some(PathBuf::from(path2).canonicalize().unwrap());
    }
}