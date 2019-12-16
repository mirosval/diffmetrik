use crate::metrics::network::get_network_metrics;
pub use crate::metrics::network::NetworkMetrics;
use cpu::get_cpu_metrics;
use cpu::CPUMetrics;
use errors::Error;
use serde::{Deserialize, Serialize};

mod cpu;
mod errors;
mod network;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTaggedMetric {
    time: std::time::Duration,
    // TODO: Replace NetworkMetric by some trait
    network: NetworkMetrics,
    cpu: CPUMetrics,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    metrics: Vec<TimeTaggedMetric>,
}

impl Metrics {
    pub fn new(metric: TimeTaggedMetric) -> Metrics {
        Metrics {
            metrics: vec![metric],
        }
    }

    pub fn merge(self, other: Metrics) -> Metrics {
        let mut metrics = self
            .metrics
            .into_iter()
            .chain(other.metrics.into_iter())
            .collect::<Vec<TimeTaggedMetric>>();
        metrics.sort_unstable_by_key({ |a| a.time });
        metrics.reverse();
        metrics.truncate(3);
        Metrics { metrics }
    }

    pub fn get_rate(&self) -> Option<MetricRate> {
        let len = self.metrics.len();
        if len > 1 {
            let m1 = self.metrics.first()?;
            let m2 = self.metrics.last()?;
            let dtime = m1.time - m2.time;
            assert!(dtime > std::time::Duration::new(1, 0));
            let rate = MetricRate {
                network: m1.network.diff(&m2.network, &dtime),
                cpu: m1.cpu,
            };
            Some(rate)
        } else {
            None
        }
    }
}

pub fn get_metrics() -> Result<Metrics> {
    let dur: std::time::Duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let network_metrics = get_network_metrics()?;
    let cpu_metrics = get_cpu_metrics()?;
    let m = TimeTaggedMetric {
        time: dur,
        network: network_metrics,
        cpu: cpu_metrics,
    };
    let metrics = Metrics::new(m);
    Ok(metrics)
}

#[derive(Debug)]
pub struct MetricRate {
    pub network: network::NetworkMetricRate,
    pub cpu: cpu::CPUMetrics,
}
