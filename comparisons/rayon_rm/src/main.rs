use std::env;

fn main() {
    rayon_rm::remove_dir_all(env::args_os().nth(1).unwrap()).unwrap();
}
