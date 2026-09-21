use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "geo-importer")]
#[command(about = "High-performance streaming geospatial data importer for OpenStreetMap & Custom Data")]
pub struct CliArgs {
    /// Path to the .osm.pbf file (e.g. data/iran-latest.osm.pbf)
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Batch size for bulk database inserts
    #[arg(short, long, default_value_t = 500)]
    pub batch_size: usize,

    /// Seed realistic opportunities & companies across Iranian cities for testing
    #[arg(long, default_value_t = false)]
    pub seed_samples: bool,

    /// Clean and remove all seeded test data from the database
    #[arg(long, default_value_t = false)]
    pub clean_samples: bool,

    /// PostgreSQL Database URL
    #[arg(long)]
    pub database_url: Option<String>,
    /// Seed 1500+ heavy test opportunities across Tehran neighborhoods
    #[arg(long, default_value_t = false)]
    pub seed_heavy_tehran: bool,
}