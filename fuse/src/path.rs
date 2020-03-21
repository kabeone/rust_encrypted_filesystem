use crate::fs::FILE_ENTRIES;
use crate::get_fs::FileInfo;
use std::ffi::OsString;


fn get_entry_path_rec(string :&mut String, infos :&FileInfo, name :&OsString) {

    if infos.parent_inode != 0 {
        unsafe {
            for entry in FILE_ENTRIES.iter() {
                if entry.attribute.ino == infos.parent_inode {
                    get_entry_path_rec(string, entry, &infos.name)
                }
            }
        }
    }
    string.push('/');
    string.push_str(name.to_str().unwrap());
}

pub fn get_entry_path(string :&mut String, parent_inode :u64, name :&OsString) {
    unsafe {
        if parent_inode == 0 {
            get_entry_path_rec(string, FILE_ENTRIES.get(0).unwrap(), name);
        } else {
            for entry in FILE_ENTRIES.iter() {
                if entry.attribute.ino == parent_inode {
                    get_entry_path_rec(string, entry, name)
                }
            }
        }
    }
}