extern crate cc;

use std::path;

fn main() {
    if !path::Path::new("odpi/include/dpi.h").exists() {
        println!("The odpi submodule isn't initialized. Run the following commands.");
        println!("  git submodule init");
        println!("  git submodule update");
        std::process::exit(1);
    }

    cc::Build::new()
        .file("odpi/embed/dpi.c")
        .include("odpi/include")
        .flag_if_supported("-Wno-unused-parameter")
        .compile("libodpic.a");
}
