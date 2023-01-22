use std::env;

fn main() {
    let mut args = env::args_os().skip(1);
    cp_stdlib::copy_dir(args.next().unwrap(), args.next().unwrap()).unwrap();
}
