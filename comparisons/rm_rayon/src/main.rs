use std::env;

fn main() {
    rm_rayon::remove_dir_all(env::args_os().nth(1).unwrap()).unwrap();
}
