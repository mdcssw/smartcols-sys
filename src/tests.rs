use core::ffi::CStr;
use core::ptr;

#[test]
fn scols_get_library_version() {
    let mut version = ptr::null();
    let r = unsafe { super::scols_get_library_version(&mut version) };
    assert!(r > 0);

    assert!(!version.is_null());
    let version = unsafe { CStr::from_ptr(version) };
    assert_ne!(0, version.count_bytes());
}
