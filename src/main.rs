mod cli;
mod metrics;
mod storage;

fn main() {
    let opt = cli::opt_from_args();
    let s = storage::Storage::new();
    let old_metrics = s.read().ok();
    let metrics = metrics::get_metrics().ok();
    s.write(&metrics).unwrap();

    match (old_metrics, metrics) {
        (Some(old), Some(new)) => {
            let diffed = new.diff(&old);

            match opt.metric {
                cli::Metric::Download => println!("D: {}", diffed.network.ibyte_rate),
                cli::Metric::Upload => println!("U: {}", diffed.network.obyte_rate),
            }
        }
        (_, _) => {
            println!("Not enough data");
        }
    }
}
