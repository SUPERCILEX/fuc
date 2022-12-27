#![allow(clippy::multiple_crate_versions)]

use std::env;

fn main() {
    let mut args = env::args().skip(1);
    fcp::fcp(&[args.next().unwrap(), args.next().unwrap()]);
}
