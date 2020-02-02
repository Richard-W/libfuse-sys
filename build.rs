extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::iter;
use std::path::PathBuf;

const FUSE_DEFAULT_API_VERSION: u32 = 26;

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

fn generate_fuse_bindings(header: &str, api_version: u32, fuse_lib: &pkg_config::Library) {
    // Find header file
    let mut header_path: Option<PathBuf> = None;
    for include_path in fuse_lib.include_paths.iter() {
        let test_path = include_path.join(header);
        if test_path.exists() {
            header_path = Some(test_path);
            break;
        }
    }
    let header_path = header_path
        .unwrap_or_else(|| panic!("Cannot find {}", header))
        .to_str()
        .unwrap_or_else(|| panic!("Path to {} contains invalid unicode characters", header))
        .to_string();

    // Gather fuse defines
    let defines = fuse_lib.defines.iter().map(|(key, val)| match val {
        Some(val) => format!("-D{}={}", key, val),
        None => format!("-D{}", key),
    });
    // Gather include paths
    let includes = fuse_lib
        .include_paths
        .iter()
        .map(|dir| format!("-I{}", dir.display()));
    // API version definition
    let api_define = iter::once(format!("-DFUSE_USE_VERSION={}", api_version));
    // Chain compile flags
    let compile_flags = defines.chain(includes).chain(api_define);

    // Create bindgen builder
    let builder = bindgen::builder();
    // Add clang flags
    let builder = builder.clang_args(compile_flags);
    // Derive Debug, Copy and Default
    let builder = builder
        .derive_default(true)
        .derive_copy(true)
        .derive_debug(true);
    // Whitelist "fuse_*" symbols and blacklist everything else
    let builder = builder
        .whitelist_recursively(false)
        .whitelist_type("^fuse.*")
        .whitelist_function("^fuse.*")
        .whitelist_var("^fuse.*");
    // Add CargoCallbacks so build.rs is rerun on header changes
    let builder = builder.parse_callbacks(Box::new(bindgen::CargoCallbacks));

    // Generate bindings
    let bindings = builder
        .header(header_path)
        .generate()
        .unwrap_or_else(|_| panic!("Failed to generate {} bindings", header));

    // Write bindings to file
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_dir.join(&header.replace(".h", ".rs"));
    bindings
        .write_to_file(&bindings_path)
        .unwrap_or_else(|_| panic!("Failed to write {}", bindings_path.display()));
}

fn main() {
    // Get the API version and panic if more than one is declared
    #[allow(unused_mut)]
    let mut api_version: Option<u32> = None;
    version!(api_version, "fuse_11", 11);
    version!(api_version, "fuse_21", 21);
    version!(api_version, "fuse_22", 22);
    version!(api_version, "fuse_24", 24);
    version!(api_version, "fuse_25", 25);
    version!(api_version, "fuse_26", 26);
    version!(api_version, "fuse_29", 29);
    version!(api_version, "fuse_30", 30);
    version!(api_version, "fuse_31", 31);
    version!(api_version, "fuse_35", 35);
    // Warn if no API version is selected
    if api_version.is_none() {
        println!(
            "cargo:warning=No FUSE API version feature selected. Defaulting to version {}.",
            FUSE_DEFAULT_API_VERSION
        );
    }
    // Fall back to default version
    let api_version = api_version.unwrap_or(FUSE_DEFAULT_API_VERSION);

    // Find libfuse
    let fuse_lib = pkg_config::Config::new()
        .probe("fuse")
        .expect("Failed to find libfuse");

    // Generate highlevel bindings
    #[cfg(feature = "fuse_highlevel")]
    generate_fuse_bindings("fuse.h", api_version, &fuse_lib);
    // Generate lowlevel bindings
    #[cfg(feature = "fuse_lowlevel")]
    generate_fuse_bindings("fuse_lowlevel.h", api_version, &fuse_lib);
}
