# libfuse-sys [![Latest Version]][crates.io] [![Build Status]][travis]

[Build Status]: https://travis-ci.org/Richard-W/libfuse-sys.svg?branch=master
[travis]: https://travis-ci.org/Richard-W/libfuse-sys
[Latest Version]: https://img.shields.io/crates/v/libfuse-sys.svg
[crates.io]: https://crates.io/crates/libfuse-sys

**Raw rust bindings to libfuse**

---

## Using libfuse-sys

Add the dependencies to your Cargo.toml
```toml
[dependencies]
libfuse-sys = { version = "*", features = ["fuse_35"] }
libc = "*"
```
You can select other API versions for fuse. Currently supported are
* `fuse_11`
* `fuse_21`
* `fuse_22`
* `fuse_24`
* `fuse_25`
* `fuse_26`
* `fuse_29`
* `fuse_30`
* `fuse_31`
* `fuse_35`

If no version is selected the default version of your installed libfuse is
used (no `FUSE_USE_VERSION` value is set).

## License

This crate itself is published under the MIT license while libfuse is published under
LGPL2+. Take special care to ensure the terms of the LGPL2+ are honored when using this
crate.
