use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CPUMetrics {
    pub m1: f32,
    pub m5: f32,
    pub m15: f32,
}

mod error;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

pub use error::CpuError;
pub use get_cpu_metrics;
