use once_cell::sync::Lazy;

static PAGE_SIZE: Lazy<usize> = Lazy::new(|| unsafe {
    let rc = libc::sysconf(libc::_SC_PAGESIZE);
    if rc == -1 {
        panic!("fail to evaluate sysconf(_SC_PAGESIZE)");
    }
    rc as usize
});

/// Returns page size which is a non zero power of 2 integer.
pub fn get() -> usize {
    *PAGE_SIZE
}
