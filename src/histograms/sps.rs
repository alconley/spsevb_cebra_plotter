use crate::utils::histogrammer::{Histogrammer};
use polars::prelude::*;

use std::f32::consts::PI;
use std::sync::Arc;
use std::path::PathBuf;

pub fn add_sps_histograms(file_paths: Arc<[PathBuf]>) -> Result<Histogrammer, PolarsError> {

        let args = ScanArgsParquet::default();

        // Load multiple parquet files
        let lf = LazyFrame::scan_parquet_files(file_paths, args)?;

        let mut h = Histogrammer::new();

    let lf = lf.with_columns(vec![
        (col("DelayFrontRightEnergy")+col("DelayFrontLeftEnergy")/ lit(2.0) ).alias("DelayFrontAverageEnergy"),
        (col("DelayBackRightEnergy")+col("DelayBackLeftEnergy")/ lit(2.0) ).alias("DelayBackAverageEnergy"),
        (col("DelayFrontLeftTime") - col("AnodeFrontTime")).alias("DelayFrontLeftTime_AnodeFrontTime"),
        (col("DelayFrontRightTime") - col("AnodeFrontTime")).alias("DelayFrontRightTime_AnodeFrontTime"),
        (col("DelayBackLeftTime") - col("AnodeFrontTime")).alias("DelayBackLeftTime_AnodeFrontTime"),
        (col("DelayBackRightTime") - col("AnodeFrontTime")).alias("DelayBackRightTime_AnodeFrontTime"),
        (col("DelayFrontLeftTime") - col("AnodeBackTime")).alias("DelayFrontLeftTime_AnodeBackTime"),
        (col("DelayFrontRightTime") - col("AnodeBackTime")).alias("DelayFrontRightTime_AnodeBackTime"),
        (col("DelayBackLeftTime") - col("AnodeBackTime")).alias("DelayBackLeftTime_AnodeBackTime"),
        (col("DelayBackRightTime") - col("AnodeBackTime")).alias("DelayBackRightTime_AnodeBackTime"),
        (col("AnodeFrontTime") - col("AnodeBackTime")).alias("AnodeFrontTime_AnodeBackTime"),
        (col("AnodeBackTime") - col("AnodeFrontTime")).alias("AnodeBackTime_AnodeFrontTime"),
        (col("AnodeFrontTime") - col("ScintLeftTime")).alias("AnodeFrontTime_ScintLeftTime"),
        (col("AnodeBackTime") - col("ScintLeftTime")).alias("AnodeBackTime_ScintLeftTime"),
        (col("DelayFrontLeftTime") - col("ScintLeftTime")).alias("DelayFrontLeftTime_ScintLeftTime"),
        (col("DelayFrontRightTime") - col("ScintLeftTime")).alias("DelayFrontRightTime_ScintLeftTime"),
        (col("DelayBackLeftTime") - col("ScintLeftTime")).alias("DelayBackLeftTime_ScintLeftTime"),
        (col("DelayBackRightTime") - col("ScintLeftTime")).alias("DelayBackRightTime_ScintLeftTime"),
        (col("ScintRightTime") - col("ScintLeftTime")).alias("ScintRightTime_ScintLeftTime"),
    ]);

    h.add_fill_hist1d_from_polars("X1", &lf, "X1", 600, (-300.0, 300.0));
    h.add_fill_hist1d_from_polars("X2", &lf, "X2", 600, (-300.0, 300.0));
    h.add_fill_hist2d_from_polars("X2_X1", &lf, "X1", 600, (-300.0, 300.0), "X2", 600, (-300.0,300.0));
    h.add_fill_hist2d_from_polars("DelayBackRight_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayBackRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackLeft_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayBackLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontRight_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayFrontRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontLeft_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayFrontLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackRight_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayBackRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackLeft_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayBackLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontRight_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayFrontRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontLeft_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayFrontLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackRight_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayBackRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackLeft_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayBackLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontRight_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayFrontRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontLeft_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayFrontLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontAverage_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayFrontAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackAverage_X1", &lf, "X1", 600, (-300.0, 300.0), "DelayBackAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontAverage_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayFrontAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackAverage_X2", &lf, "X2", 600, (-300.0, 300.0), "DelayBackAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayFrontAverage_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayFrontAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("DelayBackAverage_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "DelayBackAverageEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_ScintLeft", &lf, "ScintLeftEnergy", 256, (0.0, 4096.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_ScintLeft", &lf, "ScintLeftEnergy", 256, (0.0, 4096.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_ScintLeft", &lf, "ScintLeftEnergy", 256, (0.0, 4096.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_ScintRight", &lf, "ScintRightEnergy", 256, (0.0, 4096.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_ScintRight", &lf, "ScintRightEnergy", 256, (0.0, 4096.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_ScintRight", &lf, "ScintRightEnergy", 256, (0.0, 4096.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintLeft_X1", &lf, "X1", 600, (-300.0, 300.0), "ScintLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintLeft_X2", &lf, "X2", 600, (-300.0, 300.0), "ScintLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintLeft_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "ScintLeftEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintRight_X1", &lf, "X1", 600, (-300.0, 300.0), "ScintRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintRight_X2", &lf, "X2", 600, (-300.0, 300.0), "ScintRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("ScintRight_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "ScintRightEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_X1", &lf, "X1", 600, (-300.0, 300.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_X2", &lf, "X2", 600, (-300.0, 300.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeBack_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "AnodeBackEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_X1", &lf, "X1", 600, (-300.0, 300.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_X2", &lf, "X2", 600, (-300.0, 300.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("AnodeFront_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "AnodeFrontEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_X1", &lf, "X1", 600, (-300.0, 300.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_X2", &lf, "X2", 600, (-300.0, 300.0), "CathodeEnergy", 256, (0.0, 4096.0));
    h.add_fill_hist2d_from_polars("Cathode_Xavg", &lf, "Xavg", 600, (-300.0, 300.0), "CathodeEnergy", 256, (0.0, 4096.0));

    // Both planes histograms
    let lf_bothplanes = lf.clone().filter(col("X1").neq(lit(-1e6))).filter(col("X2").neq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("X1_bothplanes", &lf_bothplanes, "X1", 600, (-300.0, 300.0));
    h.add_fill_hist1d_from_polars("X2_bothplanes", &lf_bothplanes, "X2", 600, (-300.0, 300.0));
    h.add_fill_hist1d_from_polars("Xavg_bothplanes", &lf_bothplanes, "Xavg", 600, (-300.0, 300.0));

    h.add_fill_hist2d_from_polars("Theta_Xavg_bothplanes", &lf_bothplanes, "Xavg", 600, (-300.0, 300.0), "Theta", 300, (0.0, (PI/2.0).into()));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeFrontTime_bothplanes", &lf_bothplanes, "DelayFrontLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeFrontTime_bothplanes", &lf_bothplanes, "DelayFrontRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeBackTime_bothplanes", &lf_bothplanes, "DelayBackLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeBackTime_bothplanes", &lf_bothplanes, "DelayBackRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    
    // Only 1 plane: X1
    let lf_only_x1_plane = lf.clone().filter(col("X1").neq(lit(-1e6))).filter(col("X2").eq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("X1_only1plane", &lf_only_x1_plane, "X1", 600, (-300.0, 300.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayFrontLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayFrontRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayBackLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeFrontTime_noX2", &lf_only_x1_plane, "DelayBackRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayFrontLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayFrontRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayBackLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeBackTime_noX2", &lf_only_x1_plane, "DelayBackRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));

    // Only 1 plane: X2
    let lf_only_x2_plane = lf.clone().filter(col("X2").neq(lit(-1e6))).filter(col("X1").eq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("X2_only1plane", &lf_only_x2_plane, "X2", 600, (-300.0, 300.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayFrontLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayFrontRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayBackLeftTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeFrontTime_noX1", &lf_only_x2_plane, "DelayBackRightTime_AnodeFrontTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontLeftTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayFrontLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayFrontRightTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayFrontRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackLeftTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayBackLeftTime_AnodeBackTime", 8000, (-4000.0, 4000.0));
    // h.add_fill_hist1d_from_polars("DelayBackRightTime_relTo_AnodeBackTime_noX1", &lf_only_x2_plane, "DelayBackRightTime_AnodeBackTime", 8000, (-4000.0, 4000.0));

    // Time relative to Back Anode

    let lf_time_rel_backanode = lf.clone().filter(col("AnodeBackTime").neq(lit(-1e6))).filter(col("ScintLeftTime").neq(lit(-1e6)));

    h.add_fill_hist1d_from_polars("AnodeFrontTime_AnodeBackTime", &lf_time_rel_backanode, "AnodeFrontTime_AnodeBackTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("AnodeBackTime_AnodeFrontTime", &lf_time_rel_backanode, "AnodeBackTime_AnodeFrontTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("AnodeFrontTime_ScintLeftTime", &lf_time_rel_backanode, "AnodeFrontTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("AnodeBackTime_ScintLeftTime", &lf_time_rel_backanode, "AnodeBackTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayFrontLeftTime_ScintLeftTime", &lf_time_rel_backanode, "DelayFrontLeftTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayFrontRightTime_ScintLeftTime", &lf_time_rel_backanode, "DelayFrontRightTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayBackLeftTime_ScintLeftTime", &lf_time_rel_backanode, "DelayBackLeftTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("DelayBackRightTime_ScintLeftTime", &lf_time_rel_backanode, "DelayBackRightTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist1d_from_polars("ScintRightTime_ScintLeftTime", &lf_time_rel_backanode, "ScintRightTime_ScintLeftTime", 1000, (-3000.0 ,3000.0));
    h.add_fill_hist2d_from_polars("ScintTimeDif_Xavg", &lf_time_rel_backanode, "Xavg", 600, (-300.0, 300.0), "ScintRightTime_ScintLeftTime", 12800, (-3200.0, 3200.0));

    Ok(h)
}