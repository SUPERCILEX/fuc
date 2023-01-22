use std::env;

fn main() {
    remove_dir_all::remove_dir_all(env::args_os().nth(1).unwrap()).unwrap();
}
