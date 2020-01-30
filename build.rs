extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path;

macro_rules! version {
    ($version_var:ident, $feature:literal, $version:literal) => {
        #[cfg(feature = $feature)]
        {
            if $version_var.is_some() {
                panic!("More than one FUSE API version feature is enabled");
            }
            $version_var = Some($version);
        }
    };
}

fn main() {
    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());

    // Get the API version and panic if more than one is declared
    let mut api_version: Option<u32> = None;
    version!(api_version, "fuse_11", 11);
    version!(api_version, "fuse_21", 21);
    version!(api_version, "fuse_22", 22);
    version!(api_version, "fuse_25", 25);
    version!(api_version, "fuse_26", 26);
    let api_version = api_version.expect("No FUSE API version feature enabled");

    // Find libfuse
    let fuse_lib = pkg_config::Config::new()
        .probe("fuse")
        .expect("Failed to find libfuse");

    // Find fuse.h header
    let mut fuse_header_path: Option<path::PathBuf> = None;
    for include_path in fuse_lib.include_paths.iter() {
        let test_path = include_path.join("fuse.h");
        if test_path.exists() {
            fuse_header_path = Some(test_path);
            break;
        }
    }
    let fuse_header_path = fuse_header_path.expect("Cannot find fuse.h");

    // Generate bindings
    let include_flags = fuse_lib
        .include_paths
        .iter()
        .map(|path| format!("-I{}", path.display()));
    let define_flags = fuse_lib.defines.iter().map(|(key, val)| match val {
        Some(val) => format!("-D{}={}", key, val),
        None => format!("-D{}", key),
    });
    let bindings = bindgen::builder()
        .header(fuse_header_path.to_str().unwrap())
        .whitelist_recursively(false)
        .whitelist_type("^fuse.*")
        .whitelist_function("^fuse.*")
        .whitelist_var("^fuse.*")
        .derive_default(true)
        .derive_copy(true)
        .derive_debug(true)
        .clang_args(include_flags)
        .clang_args(define_flags)
        .clang_arg(format!("-DFUSE_USE_VERSION={}", api_version))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Failed to generate FUSE bindings");
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write FUSE bindings");
}
