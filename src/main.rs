mod cli;
mod metrics;
mod storage;

fn main() {
    let opt = cli::opt_from_args();
    let s = storage::Storage::new(opt.debug);
    let old_metrics: Option<metrics::Metrics> = {
        match s.read() {
            Ok(m) => Some(m),
            Err(_) => {
                s.reset().unwrap();
                None
            }
        }
    };
    let metrics = metrics::get_metrics().ok();

    match (old_metrics, metrics) {
        (Some(old), Some(new)) => {
            let metrics = old.merge(new);
            // dbg!(&metrics);
            s.write(&metrics).expect("aaa");
            let metric_rate: Option<metrics::MetricRate> = metrics.get_rate();
            match metric_rate {
                Some(r) => match opt.metric {
                    cli::Metric::Download => println!("D: {}", r.network.ibyte_rate),
                    cli::Metric::Upload => println!("U: {}", r.network.obyte_rate),
                },
                None => {
                    s.write(&metrics).expect("aaa");
                    println!("Not enough data");
                }
            }
        }
        (_, metrics) => {
            s.write(&metrics).expect("aaa");
            println!("Not enough data");
        }
    }
}
