use crate::metrics::network::get_network_metrics;
pub use crate::metrics::network::NetworkMetrics;
use serde::{Deserialize, Serialize};

mod network;

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub since_unix_epoch: std::time::Duration,
    pub network: NetworkMetrics,
}

impl Metrics {
    pub fn diff(&self, old: &Metrics) -> MetricRate {
        let dtime = self.since_unix_epoch - old.since_unix_epoch;
        MetricRate {
            network: self.network.diff(&old.network, &dtime),
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
        Ok(network_metrics) => Ok(Metrics {
            since_unix_epoch: dur,
            network: network_metrics,
        }),
        Err(e) => Err(MetricsError { message: e.message }),
    }
}

#[derive(Debug)]
pub struct MetricRate {
    pub network: network::NetworkMetricRate,
}
