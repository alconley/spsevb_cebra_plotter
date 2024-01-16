use crate::utils::histogrammer::{Histogrammer};
use polars::prelude::*;

use std::sync::Arc;
use std::path::PathBuf;

pub fn add_histograms(file_paths: Arc<[PathBuf]>) -> Result<Histogrammer, PolarsError> {

    let args = ScanArgsParquet::default();

    // Load multiple parquet files
    let lf = LazyFrame::scan_parquet_files(file_paths, args)?;

    let mut h = Histogrammer::new();

    // create a new column
    let lf = lf.with_columns(vec![
        (col("DelayFrontRightEnergy")+col("DelayFrontLeftEnergy")/ lit(2.0) ).alias("DelayFrontAverageEnergy"),
        (col("DelayBackRightEnergy")+col("DelayBackLeftEnergy")/ lit(2.0) ).alias("DelayBackAverageEnergy"),
    ]);

    // filter a dataframe
    let lf_bothplanes = lf.clone().filter(col("X1").neq(lit(-1e6))).filter(col("X2").neq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("Xavg_bothplanes", &lf_bothplanes, "Xavg", 600, (-300.0, 300.0));
    h.add_fill_hist2d_from_polars("AnodeBack_ScintLeft", &lf_bothplanes, "ScintLeftEnergy", 512, (0.0, 4096.0), "AnodeBackEnergy", 512, (0.0, 4096.0));

    Ok(h)
}