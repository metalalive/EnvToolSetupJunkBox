use std::{env, process};
use minigrep::Config;

fn main() {
    // args[0] is the path to the executable binary
    // move the ownership of `args` to `build()`
    let cfg = Config::build(env::args()).unwrap_or_else(
        |e| {
        println!("problem parsing argument: {e}");
        process::exit(1);
    });
    if let Err(e) = minigrep::run(&cfg) {
        println!("application error, path: {}, error:{:?}",
                 cfg.file_path, e);
        process::exit(2);
    };
}
