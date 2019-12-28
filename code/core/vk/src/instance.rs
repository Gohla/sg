use ash::Instance;

pub struct VkInstance {
  pub instance: Instance
}

impl VkInstance {
  pub(crate) fn new(instance: Instance) -> Self {
    Self { instance }
  }
}
