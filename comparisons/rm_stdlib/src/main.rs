use std::{env, fs};

fn main() {
    fs::remove_dir_all(env::args_os().nth(1).unwrap()).unwrap();
}
