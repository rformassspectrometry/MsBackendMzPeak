use std::{collections::HashMap, path};
use extendr_api::prelude::*;
use mzdata::prelude::SpectrumLike;
use mzpeak_prototyping::{MzPeakReader, archive::{ArchiveReader, DispatchArchiveSource, MzPeakArchiveType}};
use parquet::encryption::decrypt::FileDecryptionProperties;

/// Read an mzPeak archive and return the number of spectra and total number of points.
///
/// @param filename Path to the mzPeak archive.
/// @param encryption_key Optional AES decryption key (16, 24, or 32 bytes).
/// @export
pub fn load_mzpeak(filename: String, encryption_key: Option<String>) -> extendr_api::Result<Robj> {
    let mut dec_props = HashMap::default();
    if let Some(key) = encryption_key.as_ref() {
        let dec = FileDecryptionProperties::builder(key.as_bytes().to_vec())
            .build()
            .map_err(|e| extendr_api::Error::from(e.to_string()))?;
        dec_props.insert(MzPeakArchiveType::SpectrumDataArrays.tag_file_suffix().to_string(), dec.clone());
        dec_props.insert(MzPeakArchiveType::SpectrumMetadata.tag_file_suffix().to_string(), dec.clone());
        dec_props.insert(MzPeakArchiveType::SpectrumPeakDataArrays.tag_file_suffix().to_string(), dec.clone());
        dec_props.insert(MzPeakArchiveType::ChromatogramDataArrays.tag_file_suffix().to_string(), dec.clone());
        dec_props.insert(MzPeakArchiveType::ChromatogramMetadata.tag_file_suffix().to_string(), dec.clone());
        dec_props.insert(MzPeakArchiveType::WavelengthSpectrumMetadata.tag_file_suffix().to_string(), dec.clone());
        dec_props.insert(MzPeakArchiveType::WavelengthSpectrumDataArrays.tag_file_suffix().to_string(), dec.clone());
    }

    let path = path::PathBuf::from(filename);
    let archive = ArchiveReader::<DispatchArchiveSource>::from_path_with_decryption(
        path.clone(),
        dec_props,
    )
    .map_err(|e| extendr_api::Error::from(e.to_string()))?;
    let reader = MzPeakReader::from_archive_reader(archive, Some(path))
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;
    let mut n_spectra = 0i64;
    let mut n_points = 0i64;
    for spec in reader {
        n_spectra += 1;
        match spec.raw_arrays() {
            Some(arrays) => match arrays.mzs() {
                Ok(arr) => {
                    n_points += arr.len() as i64;
                    let ints = arrays.intensities()
                        .map_err(|e| extendr_api::Error::from(e.to_string()))?;
                    if arr.len() != ints.len() {
                        return Err(extendr_api::Error::from(format!(
                            "{} had {} m/z values and {} intensities",
                            spec.index(),
                            arr.len(),
                            ints.len()
                        )));
                    }
                }
                Err(e) => {
                    rprintln!(
                        "Failed to retrieve arrays for spectrum {}: {e}",
                        spec.index()
                    );
                }
            },
            None => {
                rprintln!(
                    "No raw arrays for spectrum {}",
                    spec.index()
                );
            }
        }
    }
    Ok(list!(n_spectra = n_spectra, n_points = n_points).into_robj())
}
