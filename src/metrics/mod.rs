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
    pub fn diff(&self, old: &Metrics) -> Metrics {
        dbg!(old.since_unix_epoch);
        dbg!(self.since_unix_epoch);
        dbg!(self.since_unix_epoch - old.since_unix_epoch);
        Metrics {
            since_unix_epoch: self.since_unix_epoch - old.since_unix_epoch,
            network: NetworkMetrics {
                total_ibytes: self.network.total_ibytes - old.network.total_ibytes,
                total_obytes: self.network.total_obytes - old.network.total_obytes,
            },
        }
    }
}

#[derive(Debug)]
pub struct MetricsError {
    message: String,
}

pub fn get_metrics() -> Result<Metrics, MetricsError> {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("aaa");
    dbg!(&dur);

    let network_metrics = get_network_metrics();
    match network_metrics {
        Ok(network_metrics) => Ok(Metrics {
            since_unix_epoch: dur,
            network: network_metrics,
        }),
        Err(e) => Err(MetricsError { message: e.message }),
    }
}
