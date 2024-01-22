use crate::utils::histogrammer::Histogrammer;
use polars::prelude::*;

use std::sync::Arc;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

use serde_json;

use crate::utils::cut::Cut2D;
use crate::utils::egui_polygon::EditableEguiPolygon;

pub fn add_histograms(file_paths: Arc<[PathBuf]>, cut_file_path: Option<PathBuf>) -> Result<Histogrammer, PolarsError> {
    
    let args = ScanArgsParquet::default();

    // Load multiple parquet files
    let lf = LazyFrame::scan_parquet_files(file_paths, args)?;

    let lf = if let Some(cut_path) = cut_file_path {
        cut_file_to_df(&cut_path, &lf)?
    } else {
        lf.clone() // clone lf to ensure it is returned as a LazyFrame
    };

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


fn cut_file_to_df(cut_file_path: &PathBuf, lf: &LazyFrame) -> Result<LazyFrame, polars::error::PolarsError> {

    let file = File::open(cut_file_path)?;
    let reader = BufReader::new(file);

    // let loaded_polygon: EditableEguiPolygon = serde_json::from_reader(reader)?;

    let loaded_polygon: EditableEguiPolygon = serde_json::from_reader(reader)
        .map_err(|e| PolarsError::ComputeError(format!("Failed to deserialize cut: {}", e).into()))?;

    // Extract the column names, handle the case where they are None
    let x_col = "ScintLeftEnergy";

    let y_col = "AnodeBackEnergy";

    let cut2d = Cut2D::new(loaded_polygon);

    let df = cut2d.filter_dataframe(lf, x_col, y_col)?;

    Ok(df)
}