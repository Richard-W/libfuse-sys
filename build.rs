extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::iter;
use std::path::PathBuf;

const FUSE_DEFAULT_API_VERSION: u32 = 30;

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

fn fuse_binding_filter(builder: bindgen::Builder) -> bindgen::Builder {
    let mut builder = builder
        // Whitelist "fuse_*" symbols and blocklist everything else
        .allowlist_recursively(false)
        .allowlist_type("(?i)^fuse.*")
        .allowlist_function("(?i)^fuse.*")
        .allowlist_var("(?i)^fuse.*")
        .blocklist_type("fuse_log_func_t")
        .blocklist_function("fuse_set_log_func");
    // TODO: properly bind fuse_log_func_t and allowlist fuse_set_log_func again

    if cfg!(target_os = "macos") {
        // osxfuse needs this type
        builder = builder.allowlist_type("setattr_x");
    }
    builder
}

fn cuse_binding_filter(builder: bindgen::Builder) -> bindgen::Builder {
    builder
        // Whitelist "cuse_*" symbols and blocklist everything else
        .allowlist_recursively(false)
        .allowlist_type("(?i)^cuse.*")
        .allowlist_function("(?i)^cuse.*")
        .allowlist_var("(?i)^cuse.*")
}

fn generate_fuse_bindings(
    header: &str,
    api_version: u32,
    fuse_lib: &pkg_config::Library,
    binding_filter: fn(bindgen::Builder) -> bindgen::Builder,
) {
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
    let mut builder = bindgen::builder()
        // Add clang flags
        .clang_args(compile_flags)
        // Derive Debug, Copy and Default
        .derive_default(true)
        .derive_copy(true)
        .derive_debug(true)
        // Add CargoCallbacks so build.rs is rerun on header changes
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    builder = binding_filter(builder);

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

    let mut pkgcfg = pkg_config::Config::new();
    pkgcfg.cargo_metadata(false);

    // Find libfuse
    let try_fuse_lib = pkgcfg.probe("fuse");
    let try_fuse3_lib = pkgcfg.probe("fuse3");
    let fuse_lib = match (try_fuse_lib, try_fuse3_lib) {
        (Err(err), Err(err3)) => panic!(
            "Failed to find pkg-config modules fuse ({}) or fuse3 ({})",
            err, err3
        ),
        (Ok(_), Err(_)) => "fuse",
        (Err(_), Ok(_)) => "fuse3",
        (Ok(_), Ok(_)) => {
            // Strange situation but we should just try to find the module that is more likely
            // to be the correct one here.
            if api_version < 30 {
                "fuse"
            } else {
                "fuse3"
            }
        }
    };
    let fuse_lib = pkgcfg.cargo_metadata(true).probe(fuse_lib).unwrap();

    // Generate highlevel bindings
    #[cfg(feature = "fuse_highlevel")]
    generate_fuse_bindings("fuse.h", api_version, &fuse_lib, fuse_binding_filter);
    // Generate lowlevel bindings
    #[cfg(feature = "fuse_lowlevel")]
    generate_fuse_bindings(
        "fuse_lowlevel.h",
        api_version,
        &fuse_lib,
        fuse_binding_filter,
    );
    // Generate lowlevel cuse bindings
    #[cfg(feature = "cuse_lowlevel")]
    generate_fuse_bindings(
        "cuse_lowlevel.h",
        api_version,
        &fuse_lib,
        cuse_binding_filter,
    );
}
