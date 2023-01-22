use std::env;

fn main() {
    rm_og_crappy::remove_dir_all(env::args_os().nth(1).unwrap()).unwrap();
}
