use std::env;
use std::fs;
use std::process::exit;
use std::path::Path;

fn usage() {
    println!("\nUsage: caledfsw1tch path_to_encrypted path_to_mountpoint");
    println!("\n      in case \"path_to_encrypted\" doesn't exist it's gonna be created, same goes for \"path_to_moountpoint\"");
    exit(84);
}

fn help() {
    println!("\nprint some fkin help\n");
    exit(0);
}

pub fn check_args() {
    if env::args_os().len() != 3 {
        if env::args_os().len() == 2 && env::args_os().nth(1).unwrap().eq("--help") {
            help();
        }
        usage();
    }

    match fs::metadata(env::args_os().nth(2).unwrap()) {
        Ok(t) => {
           if !t.is_dir() {
               println!("Error: \"{}\" is not a directory", env::args_os().nth(2).unwrap().to_str().unwrap());
               exit(84);
           }
        },
        Err(e) => {
            println!("Error: \"{}\": {}", env::args_os().nth(2).unwrap().to_str().unwrap(), e);
            exit(84);
        }
    }

    match fs::metadata(env::args_os().nth(1).unwrap()) {
        Ok(t) => {
            if !t.is_dir() {
                println!("Error: \"{}\" not a directory", env::args_os().nth(1).unwrap().to_str().unwrap());
                exit(84);
            }
        }
        Err(_e) => {
            match fs::create_dir_all(Path::new(&env::args_os().nth(1).unwrap())) {
                Ok(_t) => (),
                Err(e) => println!("Error: {}", e)
            }
        }
    }
}