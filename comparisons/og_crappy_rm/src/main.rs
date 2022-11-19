use std::env;

fn main() {
    og_crappy_rm::remove_dir_all(env::args_os().nth(1).unwrap()).unwrap();
}
