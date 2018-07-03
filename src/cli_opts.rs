
#[derive(Debug, StructOpt)]
#[structopt(name = "datagen", about = "Generate random data sets")]
pub struct CliOptions {
    /// Enable debug logging to stderr. Multiple occurrences will increase the verbosity
    #[structopt(short = "V", parse(from_occurrences))]
    pub debug: u64,

    /// number of iterations to print
    #[structopt(short = "n", long = "iterations")]
    pub iteration_count: u64,

    /// Specification of how to generate data for a column
    #[structopt(short = "g", long = "generate")]
    pub program: String,
}
