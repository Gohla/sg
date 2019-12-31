use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::c_char;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("One or more required items are missing: {0:?}")]
pub struct MissingError(pub Vec<CString>);

pub fn get_enabled_or_missing<I: IntoIterator<Item=CString>>(available: I, wanted: &HashSet<CString>, required: &HashSet<CString>)
  -> Result<(HashSet<CString>, Vec<*const c_char>), MissingError> {
  let available: HashSet<_> = available.into_iter().collect();
  let missing: Vec<_> = required.difference(&available).cloned().collect();
  if !missing.is_empty() {
    return Err(MissingError(missing));
  }
  let enabled: HashSet<_> = available.intersection(&wanted.union(&required).cloned().collect()).cloned().collect();
  let raw: Vec<_> = enabled.iter().map(|n| n.as_ptr()).collect();
  Ok((enabled, raw))
}
