use std::ops::Deref;

use ash::{Entry as VkEntry, LoadingError};
use thiserror::Error;

use crate::version::VkVersion;

// Wrapper

pub struct Entry {
  pub wrapped: VkEntry,
}

// Creation

#[derive(Error, Debug)]
pub enum EntryCreateError {
  #[error("Failed to load Vulkan library")]
  LoadError(#[from] LoadingError),
}

impl Entry {
  pub fn new() -> Result<Self, EntryCreateError> {
    let wrapped = VkEntry::new()?;
    Ok(Self { wrapped })
  }
}

// API

impl Entry {
  pub fn instance_version(&self) -> Option<VkVersion> {
    match self.wrapped.try_enumerate_instance_version() {
      Ok(Some(version)) => Some(version.into()),
      Ok(None) => Some(VkVersion::default()),
      Err(_) => None,
    }
  }
}

// Implementations

impl Deref for Entry {
  type Target = VkEntry;

  #[inline]
  fn deref(&self) -> &Self::Target { &self.wrapped }
}
