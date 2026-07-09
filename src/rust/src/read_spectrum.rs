use extendr_api::prelude::*;
use std::{path};
use mzdata::prelude::*;
use mzpeak_prototyping::MzPeakReader;
use mzdata::spectrum::PeakDataLevel;


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

/// Read the peaks of a mzPeak archive, preferring pre-computed centroids
/// (spectra_peaks.mzpeak) over profile data when available.
///
/// @param reader Already-open reader (for reuse across bulk operations)
///
/// @param index spectrum index
///
/// @noRd
pub fn read_peaks_with_reader(reader: &mut MzPeakReader, index: usize) -> extendr_api::Result<Robj> {
    // Try the dedicated peaks file first.
    if let Some(level) = reader
        .get_spectrum_peaks_for(index as u64)
        .map_err(|e| extendr_api::Error::from(e.to_string()))?
    {
        return peak_level_to_df(level);
    }

    // Fall back: no explicit peaks stored for this spectrum -> pick peaks
    // from the profile data ourselves.
    let mut spec = reader
        .get_spectrum(index)
        .ok_or_else(|| extendr_api::Error::from(format!("Spectrum {index} not found")))?;

    spec.pick_peaks(1.0)
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;

    let arrays = spec
        .raw_arrays()
        .ok_or_else(|| extendr_api::Error::from("No raw arrays available".to_string()))?;

    let mzs = arrays.mzs().map_err(|e| extendr_api::Error::from(e.to_string()))?;
    let ints = arrays.intensities().map_err(|e| extendr_api::Error::from(e.to_string()))?;

    let mz_vec: Vec<f64> = mzs.iter().copied().collect();
    let int_vec: Vec<f64> = ints.iter().map(|x| *x as f64).collect();

    Ok(data_frame!(mz = mz_vec, intensity = int_vec).into_robj())
}

/// Read the peaks of a mzPeak archive, preferring pre-computed centroids
/// (spectra_peaks.mzpeak) over profile data when available.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param index `integer` with the index of the spectrum to extract.
///
/// @noRd
pub fn read_peaks(filename: String, index: usize) -> extendr_api::Result<Robj> {
    let path = path::PathBuf::from(filename);
    let mut reader = MzPeakReader::new(path)
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;
    read_peaks_with_reader(&mut reader, index)
}

/// Convert a `PeakDataLevel` into an (mz, intensity) data frame.
fn peak_level_to_df<C, D>(level: PeakDataLevel<C, D>) -> extendr_api::Result<Robj>
where
    C: CentroidLike,
    D: DeconvolutedCentroidLike,
{
    match level {
        PeakDataLevel::Centroid(peaks) => {
            let mz_vec: Vec<f64> = peaks.iter().map(|p| p.mz()).collect();
            let int_vec: Vec<f64> = peaks.iter().map(|p| p.intensity() as f64).collect();
            Ok(data_frame!(mz = mz_vec, intensity = int_vec).into_robj())
        }
        PeakDataLevel::Deconvoluted(peaks) => {
            let mz_vec: Vec<f64> = peaks.iter().map(|p| p.neutral_mass()).collect();
            let int_vec: Vec<f64> = peaks.iter().map(|p| p.intensity() as f64).collect();
            Ok(data_frame!(mz = mz_vec, intensity = int_vec).into_robj())
        }
        _ => Err(extendr_api::Error::from(
            "No centroided peak data available for this spectrum".to_string(),
        )),
    }
}
