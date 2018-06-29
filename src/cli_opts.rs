use structopt::StructOpt;


#[derive(Debug, StructOpt)]
#[structopt(name = "datagen", about = "Generate random data sets")]
pub struct CliOptions {
    /// Enable debug logging to stderr
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// Specification of how to generate data for a column
    #[structopt(short = "c", long = "column")]
    pub column_specs: Vec<String>,
}
