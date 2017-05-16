extern crate gcc;

use std::fs;
use std::path;

fn main() {
    if !path::Path::new("odpi/include/dpi.h").exists() {
        println!("The odpi submodule isn't initialized. Run the following commands.");
        println!("  git submodule init");
        println!("  git submodule update");
        std::process::exit(1);
    }

    let mut cfg = gcc::Config::new();
    for entry in fs::read_dir("odpi/src").unwrap() {
        let fname = entry.unwrap().file_name().into_string().unwrap();
        if fname.ends_with(".c") {
            cfg.file(format!("odpi/src/{}", fname));
        }
    }
    cfg.include("odpi/include")
        .include("odpi/src")
        .compile("libodpic.a");
}
