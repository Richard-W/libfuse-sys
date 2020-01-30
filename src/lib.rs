#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use libc::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Main function of FUSE
///
/// Implemented as a macro in the original fuse header.
pub unsafe fn fuse_main(
    argc: c_int,
    argv: *mut *mut c_char,
    op: *const fuse_operations,
    user_data: *mut c_void,
) -> c_int {
    fuse_main_real(argc, argv, op, std::mem::size_of_val(&*op), user_data)
}
