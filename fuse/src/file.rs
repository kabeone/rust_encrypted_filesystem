use crate::path::get_entry_path;

pub struct FileHandle {
    pub handle :u64,
    pub path : String
}

pub static mut FILE_HANDLES : Vec<FileHandle> = Vec::new();

fn get_new_handle() -> u64 {
    let mut handle :u64 = 0;
    let mut flag = 1;

    unsafe {
        while flag != 0 {
            handle += 1;
            flag = 0;

            for entry in FILE_HANDLES.iter() {
                if entry.handle == handle {
                    flag += 1;
                }
            }
        }
    }
    handle
}

pub fn get_new_file_handle(inode :u64) -> u64 {
    let mut n = 0;

    unsafe {
        for entry in crate::fs::FILE_ENTRIES.iter() {
            if entry.attribute.ino == inode {

                let mut src_path = String::from(crate::fs::PATH_SRC.clone().unwrap().to_str().unwrap());
                get_entry_path(&mut src_path, entry.parent_inode, &entry.name);
                let handle = get_new_handle();
                n = handle;
                FILE_HANDLES.push(FileHandle{handle, path : src_path.clone() });
            }
        }
    }
    n
}