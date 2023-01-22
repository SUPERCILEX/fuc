use std::env;

fn main() {
    let mut args = env::args_os().skip(1);
    cp_rayon::copy_dir(args.next().unwrap(), args.next().unwrap()).unwrap();
}
