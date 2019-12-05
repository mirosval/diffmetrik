use crate::metrics::get_metrics;
use structopt::StructOpt;

mod cli;
mod metrics;
mod storage;

fn main() {
    let opt = cli::Opt::from_args();
    dbg!(&opt);

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
