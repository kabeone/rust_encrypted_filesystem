mod fs;
mod check;
mod get_fs;
mod path;
mod file;
mod setattr;

use crate::check::check_args;
use crate::get_fs::get_fs;
use std::env;


fn main() {
    check_args();
    get_fs();

    let mountpoint = env::args_os().nth(2).unwrap();
    fuse::mount(fs::FS, &mountpoint, &[]).unwrap();
}