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
    let metrics = metrics.map(|m| {
        s.write(&m).expect("aaa");
        m
    });

    match (old_metrics, metrics) {
        (Some(old), Some(new)) => {
            let metrics = old.merge(new);
            let metric_rate: Option<metrics::MetricRate> = metrics.get_rate();
            match metric_rate {
                Some(r) => match opt.metric {
                    cli::Metric::Download => println!("D: {}", r.network.ibyte_rate),
                    cli::Metric::Upload => println!("U: {}", r.network.obyte_rate),
                },
                None => {
                    println!("Not enough data");
                }
            }
        }
        (_, _) => {
            println!("Not enough data");
        }
    }
}
