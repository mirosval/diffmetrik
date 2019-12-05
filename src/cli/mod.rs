use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum Metric {
        Download,
        Upload,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "baisc")]
pub struct Opt {
    #[structopt(short, long, possible_values = &Metric::variants(), case_insensitive = true)]
    pub metric: Metric,
}

