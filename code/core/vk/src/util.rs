use std::ffi::{CString, NulError};
use std::os::raw::c_char;
use std::ptr;

#[inline]
pub fn cstring_from_str(str: Option<&str>) -> Result<(Option<CString>, *const c_char), NulError> {
  match str {
    Some(str) => {
      let cstr = CString::new(str)?;
      let ptr = cstr.as_ptr();
      Ok((Some(cstr), ptr))
    }
    None => Ok((None, ptr::null())),
  }
}

#[inline]
pub fn cstring_from_string(string: Option<String>) -> Result<(Option<CString>, *const c_char), NulError> {
  match string {
    Some(string) => {
      let cstr = CString::new(string)?;
      let ptr = cstr.as_ptr();
      Ok((Some(cstr), ptr))
    }
    None => Ok((None, ptr::null())),
  }
}
