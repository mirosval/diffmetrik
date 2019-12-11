mod cli;
mod metrics;
mod storage;

fn main() {
    let opt = cli::opt_from_args();
    let storage = storage::Storage::new(opt.debug);
    let old_metrics: Option<metrics::Metrics> = storage
        .read()
        .map_err(|e| {
            storage.reset().unwrap();
            e
        })
        .ok();
    let metrics = metrics::get_metrics().ok();
    let write_error = "Unable to write temp file with the metrics";

    match (old_metrics, metrics) {
        (Some(old), Some(new)) => {
            let metrics = old.merge(new);
            //dbg!(&metrics);
            storage.write(&metrics).expect(write_error);
            let metric_rate: Option<metrics::MetricRate> = metrics.get_rate();
            match metric_rate {
                Some(r) => match opt.metric {
                    cli::Metric::Download => println!("D: {}", r.network.ibyte_rate),
                    cli::Metric::Upload => println!("U: {}", r.network.obyte_rate),
                },
                None => {
                    storage.write(&metrics).expect(write_error);
                    println!("Not enough data");
                }
            }
        }
        (_, metrics) => {
            storage.write(&metrics).expect(write_error);
            println!("Not enough data");
        }
    }
}
