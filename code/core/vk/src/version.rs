use ash::{vk_make_version, vk_version_major, vk_version_minor, vk_version_patch};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct VkVersion {
  major: u32,
  minor: u32,
  patch: u32,
}

impl Default for VkVersion {
  fn default() -> Self {
    Self { major: 1, minor: 0, patch: 0 }
  }
}

impl From<u32> for VkVersion {
  fn from(version: u32) -> Self {
    let major = vk_version_major!(version);
    let minor = vk_version_minor!(version);
    let patch = vk_version_patch!(version);
    Self { major, minor, patch }
  }
}

impl Into<u32> for VkVersion {
  fn into(self) -> u32 {
    vk_make_version!(self.major, self.minor, self.patch)
  }
}

impl VkVersion {
  pub fn new(major: u32, minor: u32, patch: u32) -> Self {
    Self { major, minor, patch }
  }
}
