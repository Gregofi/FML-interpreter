
use core::ffi::c_void;

extern "C" {
    #[link(name="heap_init", kind="static")]
    fn heap_init();
    #[link(name="heap_alloc", kind="static")]
    fn heap_alloc(bytes: i32) -> *mut c_void;
    #[link(name="heap_free", kind="static")]
    fn heap_free(ptr: *mut c_void) -> bool;
    #[link(name="heap_done", kind="static")]
    fn heap_done(ptr: *mut c_void) -> bool;
}
