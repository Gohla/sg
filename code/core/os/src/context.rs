use winit::event_loop::EventLoop;

pub struct OsContext {
  pub(crate) event_loop: EventLoop<()>,
}

impl OsContext {
  pub fn new() -> OsContext {
    let event_loop = EventLoop::new();
    return OsContext { event_loop }
  }
}
