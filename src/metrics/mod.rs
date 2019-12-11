use crate::metrics::network::get_network_metrics;
pub use crate::metrics::network::NetworkMetrics;
use serde::{Deserialize, Serialize};

mod network;

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeTaggedMetric {
    time: std::time::Duration,
    // TODO: Replace NetworkMetric by some trait
    network: NetworkMetrics,
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
            let m1 = &self.metrics.first()?;
            let m2 = &self.metrics.last()?;
            let dtime = m1.time - m2.time;
            assert!(dtime > std::time::Duration::new(1, 0));
            let rate = MetricRate {
                network: m1.network.diff(&m2.network, &dtime),
            };
            Some(rate)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct MetricsError {
    message: String,
}

impl std::convert::From<std::time::SystemTimeError> for MetricsError {
    fn from(_e: std::time::SystemTimeError) -> MetricsError {
        MetricsError {
            message: "Given system time is in the future".to_string(),
        }
    }
}

pub fn get_metrics() -> Result<Metrics, MetricsError> {
    let dur = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    let network_metrics = get_network_metrics();
    match network_metrics {
        Ok(network_metrics) => {
            let m = TimeTaggedMetric {
                time: dur,
                network: network_metrics,
            };
            let metrics = Metrics::new(m);
            Ok(metrics)
        }
        Err(e) => Err(MetricsError { message: e.message }),
    }
}

#[derive(Debug)]
pub struct MetricRate {
    pub network: network::NetworkMetricRate,
}
