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
    #[structopt(long, default_value = "diffmetrik.json")]
    pub file_name: String,

    #[structopt(short, long, possible_values = &Metric::variants(), case_insensitive = true)]
    pub metric: Metric,

    #[structopt(long)]
    pub daemon: bool,

    #[structopt(short, long)]
    pub debug: bool,
}

pub fn opt_from_args() -> Opt {
    let opt = Opt::from_args();
    if opt.debug {
        dbg!(&opt);
    }
    opt
}
