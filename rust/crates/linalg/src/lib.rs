mod api;
mod engine;
mod render;

use std::alloc::Layout;

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    unsafe { std::alloc::alloc(Layout::from_size_align_unchecked(len, 1)) }
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        std::alloc::dealloc(ptr, Layout::from_size_align_unchecked(len, 1));
    }
}

#[no_mangle]
pub extern "C" fn solve(ptr: *mut u8, len: usize) -> *mut u8 {
    let input = if ptr.is_null() || len == 0 {
        ""
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        std::str::from_utf8(bytes).unwrap_or("")
    };
    pack(api::solve_json(input))
}

fn pack(s: String) -> *mut u8 {
    let bytes = s.into_bytes();
    let total = 4 + bytes.len();
    unsafe {
        let p = std::alloc::alloc(Layout::from_size_align_unchecked(total, 1));
        if p.is_null() {
            return p;
        }
        let len_bytes = (bytes.len() as u32).to_le_bytes();
        std::ptr::copy_nonoverlapping(len_bytes.as_ptr(), p, 4);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p.add(4), bytes.len());
        p
    }
}
