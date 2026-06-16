mod read;
mod read_spectrum;
mod read_desc;

use extendr_api::prelude::*;
use mzpeak_prototyping::MzPeakReader;
use mzdata::prelude::*;

/// Read an mzPeak archive and return the number of spectra and total number of
/// points.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param encryption_key `character(1)` Optional AES decryption key (16, 24,
///     or 32 bytes).
///
/// @noRd
#[extendr]
fn mzpeak_info(filename: String, encryption_key: Option<String>) ->
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
/// @noRd
#[extendr]
fn mzpeak_read_desc(filename: String, encryption_key: Option<String>) ->
                     extendr_api::Result<Robj> {
    let n_spectra = read::mzpeak_n_spectra(filename.clone(),
                                    encryption_key).unwrap();
    let mut descriptions = Vec::with_capacity(n_spectra);
    for i in 0..n_spectra {
        descriptions.push(read_desc::read_desc(filename.clone(), i)?);
    }
    Ok(List::from_values(descriptions).into_robj())
}

/// Read an the spectrum of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param encryption_key `character(1)` Optional AES decryption key (16, 24,
///     or 32 bytes).
///
/// @noRd
#[extendr]
fn mzpeak_read_spectrum(filename: String, encryption_key: Option<String>) ->
                         extendr_api::Result<Robj> {
    let n_spectra = read::mzpeak_n_spectra(filename.clone(),
                                    encryption_key).unwrap();
    let mut spectrum = Vec::with_capacity(n_spectra);
    for i in 0..n_spectra {
        spectrum.push(read_spectrum::read_spectrum(filename.clone(), i)?);
    }
    Ok(List::from_values(spectrum).into_robj())
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod MsBackendMzPeak;
    fn mzpeak_info;
    fn mzpeak_read_desc;
    fn mzpeak_read_spectrum;
}
