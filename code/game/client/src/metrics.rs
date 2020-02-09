use anyhow::{Context, Result};
use log::info;
use metrics_core::{Builder, Drain, Observe};
use metrics_observer_yaml::{YamlBuilder, YamlObserver};
use metrics_runtime::{Controller, Receiver};

pub struct Metrics {
  controller: Controller,
  observer: YamlObserver,
}

impl Metrics {
  pub fn new() -> Result<Metrics> {
    let metrics_receiver = Receiver::builder().build()
      .with_context(|| "Failed to initialize metrics receiver")?;
    let controller = metrics_receiver.controller();
    let observer = YamlBuilder::new().build();
    metrics_receiver.install();
    Ok(Metrics { controller, observer })
  }

  pub fn print_metrics(&mut self) {
    self.controller.observe(&mut self.observer);
    let output = self.observer.drain();
    info!("{}", output);
  }
}
