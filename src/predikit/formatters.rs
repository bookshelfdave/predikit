// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::events::ChkLifecycleEvent;

pub mod default;

pub trait OutputFormatter {
    fn init(&mut self, cfg: FormatterConfig);
    fn process_events(&mut self, receiver: std::sync::mpsc::Receiver<ChkLifecycleEvent>);
    fn term(&mut self);
}

pub struct FormatterConfig {
    pub color: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        FormatterConfig { color: true }
    }
}
