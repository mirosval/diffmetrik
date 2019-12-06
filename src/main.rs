use crate::metrics::get_metrics;

mod cli;
mod metrics;
mod storage;

fn main() {
    let opt = cli::opt_from_args();
    let s = storage::Storage::new();
    let old_metrics = s.read().ok();
    let metrics = Some(get_metrics().unwrap());

    s.write(&metrics).unwrap();

    match (old_metrics, metrics) {
        (Some(old), Some(new)) => {
            let diffed = new.diff(&old);
            println!("D: {}", diffed.network.total_ibytes);
        }
        (_, _) => {
            println!("Not enough data");
        }
    }
}
