use extendr_api::prelude::*;
use std::{path};
use mzdata::prelude::*;
use mzpeak_prototyping::MzPeakReader;

/// Read an the spectrum of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param index `integer` with the index of the spectrum to extract.
///
/// @noRd
pub fn read_spectrum(filename: String, index: usize) ->
                     extendr_api::Result<Robj> {
    let path = path::PathBuf::from(filename);
    let mut reader = MzPeakReader::new(path)
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;

    let mut spec = reader
        .get_spectrum(index)
        .ok_or_else(|| extendr_api::Error::from(format!("Spectrum {index} not found")))?;

    if let Some(arrays) = spec.raw_arrays() {
        for (k, v) in arrays.iter() {
            rprintln!("{k:?} => {:?}", v.data_len());
        }
    }

    spec.pick_peaks(1.0)
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;

    let arrays = spec
        .raw_arrays()
        .ok_or_else(|| extendr_api::Error::from("No raw arrays available".to_string()))?;

    let mzs = arrays.mzs()
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;
    let ints = arrays.intensities()
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;

    let mz_vec: Vec<f64> = mzs.iter().copied().collect();
    let int_vec: Vec<f64> = ints.iter().map(|x| *x as f64).collect();

    Ok(data_frame!(mz = mz_vec, intensity = int_vec).into_robj())
}
