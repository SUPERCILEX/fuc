use std::env;

fn main() {
    let mut args = env::args_os().skip(1);
    rayon_cp::copy_dir(args.next().unwrap(), args.next().unwrap()).unwrap();
}
