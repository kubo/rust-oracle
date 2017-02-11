extern crate gcc;

use std::env;
use std::fs;
use std::path;

fn main() {
    if !path::Path::new("odpi/include/dpi.h").exists() {
        println!("The odpi submodule isn't initialized. Run the following commands.");
        println!("  git submodule init");
        println!("  git submodule update");
        std::process::exit(1);
    }

    let target = env::var("TARGET").unwrap();
    let (oci_lib_link_name, oci_lib_real_name) = if target.contains("win32") {
        ("oci", "oci.lib")
    } else if target.contains("darwin") {
        ("clntsh", "libclntsh.dylib")
    } else {
        ("clntsh", "libclntsh.so")
    };

    let oci_inc_dir = match env::var("OCI_INC_DIR") {
        Ok(val) => val,
        Err(_) => {
            println!("Set OCI_INC_DIR environment variable to point to the directory containing Oracle header files.");
            std::process::exit(1);
        },
    };
    if !path::Path::new(&oci_inc_dir).join("ociap.h").exists() {
        println!("ociap.h could not be found in OCI_INC_DIR: {}", oci_inc_dir);
        std::process::exit(1);
    }

    let oci_lib_dir = match env::var("OCI_LIB_DIR") {
        Ok(val) => val,
        Err(_) => {
            println!("Set OCI_LIB_DIR environment variable to point to the directory containing Oracle libraries.");
            std::process::exit(1);
        },
    };
    if !path::Path::new(&oci_lib_dir).join(oci_lib_real_name).exists() {
        println!("{} could not be found in OCI_LIB_DIR: {}", oci_lib_real_name, oci_lib_dir);
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
        .include(oci_inc_dir)
        .compile("libodpic.a");
    println!("cargo:rustc-link-lib={}", oci_lib_link_name);
    println!("cargo:rustc-link-search=native={}", oci_lib_dir);
}
