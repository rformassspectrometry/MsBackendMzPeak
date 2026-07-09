mod read;
mod read_spectrum;
mod read_desc;

use read_desc::{RSpectrumDescription, read_desc_raw_with_reader};
use read_spectrum::read_peaks_with_reader;
use extendr_api::prelude::*;
use rayon::prelude::*;
use mzdata::spectrum::SpectrumDescription;
use mzpeak_prototyping::MzPeakReader;
use std::path::PathBuf;

/// Read an mzPeak archive and return the number of spectra and total number of
/// points.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param encryption_key `character(1)` Optional AES decryption key (16, 24,
///     or 32 bytes).
///
/// @return `list` containing the number of spectra and the number of points in
//      the file.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_info(filename: String,
                #[extendr(default = "NULL")] encryption_key: Option<String>) ->
                extendr_api::Result<Robj> {
    read::info(filename, encryption_key)
}

/// Function to read description field of a mzPeak file.
///
/// @param filename Path to the mzPeak archive.
///
/// @param encryption_key `character(1)` Optional AES decryption key (16, 24,
///     or 32 bytes).
///
/// @param index optional `integer` with the index of the SpectrumDescription
//      to extract. If not provided, the function extract all the
//      SpectrumDescription.
///
/// @return `list` containing the matadate of the spectra.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_desc(filename: String,
                    #[extendr(default = "NULL")] encryption_key: Option<String>,
                    #[extendr(default = "NULL")] index: Option<usize>) ->
                    extendr_api::Result<Robj> {
    let n_spectra = read::mzpeak_n_spectra(filename.clone(),
                                            encryption_key).unwrap();

    match &index {
        Some(i) => {
            if *i >= n_spectra {
                return Err(extendr_api::Error::Other(format!("No spectrum available for index {i}")));
            }
            let robj = read_desc::read_desc(filename.clone(), *i)?;
            Ok(List::from_values(vec![robj]).into_robj())
        },
        None => {
            let raw: Vec<SpectrumDescription> = (0..n_spectra)
                .into_par_iter()
                .map_init(
                    || {
                        let path = PathBuf::from(&filename);
                        MzPeakReader::new(path).expect("failed to open mzPeak reader")
                    },
                    |reader, i| read_desc_raw_with_reader(reader, i),
                )
                .collect::<Result<Vec<_>, String>>()?;

            let descriptions: Vec<Robj> = raw.into_iter()
                .map(|d| RSpectrumDescription(d).into())
                .collect();

            Ok(List::from_values(descriptions).into_robj())
        },
    }
}

/// Read an the spectrum of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param encryption_key `character(1)` Optional AES decryption key (16, 24,
///     or 32 bytes).
///
/// @param index optional `integer` with the index of the Spectrum to extract.
///     If not provided, the function extract all the Spectrum.
///
/// @return `list` of `data.frame` containing the spectra data.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_spectrum(filename: String,
                    #[extendr(default = "NULL")] encryption_key: Option<String>,
                    #[extendr(default = "NULL")] index: Option<usize>) ->
                    extendr_api::Result<Robj> {
    let n_spectra = read::mzpeak_n_spectra(filename.clone(),
                                    encryption_key).unwrap();
    let spectrum = match &index {
        Some(i) => {
            if *i >= n_spectra {
                return Err(extendr_api::Error::Other(format!("No spectrum available for index {i}")));
            }
            vec![read_spectrum::read_spectrum(filename.clone(), *i)?]
        },
        None => {
            let mut spectrum = Vec::with_capacity(n_spectra);
            for i in 0..n_spectra {
                spectrum.push(read_spectrum::read_spectrum(filename.clone(),
                                                            i)?);
            }
            spectrum
        },
    };
    Ok(List::from_values(spectrum).into_robj())
}

/// Read an the peaks of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param encryption_key `character(1)` Optional AES decryption key (16, 24,
///     or 32 bytes).
///
/// @param index optional `integer` with the index of the Spectrum to extract.
///     If not provided, the function extract all the Spectrum.
///
/// @return `list` of `data.frame` containing the spectra data.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_peaks(filename: String,
                    #[extendr(default = "NULL")] encryption_key: Option<String>,
                    #[extendr(default = "NULL")] index: Option<usize>) ->
                    extendr_api::Result<Robj> {
    let n_spectra = read::mzpeak_n_spectra(filename.clone(),
                                    encryption_key).unwrap();
    let peaks = match &index {
        Some(i) => {
            if *i >= n_spectra {
                return Err(extendr_api::Error::Other(format!("No peaks available for index {i}")));
            }
            vec![read_spectrum::read_peaks(filename.clone(), *i)?]
        },
        None => {
            // Open reader once and reuse across all spectra for efficiency
            let path = PathBuf::from(&filename);
            let mut reader = MzPeakReader::new(path)
                .map_err(|e| extendr_api::Error::from(e.to_string()))?;

            let mut peaks = Vec::with_capacity(n_spectra);
            for i in 0..n_spectra {
                peaks.push(read_peaks_with_reader(&mut reader, i)?);
            }
            peaks
        },
    };
    Ok(List::from_values(peaks).into_robj())
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod MsBackendMzPeak;
    fn mzpeak_info;
    fn mzpeak_read_desc;
    fn mzpeak_read_spectrum;
    fn mzpeak_read_peaks;
}
