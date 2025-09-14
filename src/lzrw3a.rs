use libc::{c_int, c_void, free, size_t};
use std::slice;

#[link(name = "lzrw3a")]
unsafe extern "C" {
    fn lzrw3a_c(action: c_int, buffer: *const u8, size: size_t, out_size: *mut c_void) -> *mut u8;
}

#[derive(PartialEq, Eq)]
pub enum CompressAction {
    Decompress,
    Compress,
}

impl CompressAction {
    fn as_c_int(&self) -> c_int {
        match self {
            CompressAction::Compress => 1,
            CompressAction::Decompress => 2,
        }
    }
}

impl std::fmt::Display for CompressAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressAction::Decompress => write!(f, "Decompress"),
            CompressAction::Compress => write!(f, "Compress"),
        }
    }
}

pub fn compress(action: CompressAction, buffer: &[u8]) -> Option<Vec<u8>> {
    let mut size_out: i32 = 0;

    unsafe {
        let ptr = lzrw3a_c(
            action.as_c_int(),
            buffer.as_ptr(),
            buffer.len(),
            &mut size_out as *mut i32 as *mut c_void,
        );

        if ptr.is_null() {
            return None;
        }

        // create vector and cleanup
        let vec = slice::from_raw_parts(ptr, size_out as usize).to_vec();
        free(ptr as *mut c_void);

        Some(vec)
    }
}
