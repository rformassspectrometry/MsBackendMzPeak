mod read;
mod read_spectrum;

use extendr_api::prelude::*;
use mzpeak_prototyping::MzPeakReader;
use mzdata::prelude::*;

/// Basic function to try read a small mzpeak.
///
/// @noRd
#[extendr]
fn mz_peak_reader_test() -> extendr_api::Result<()> {
    let mut reader = MzPeakReader::new("/home/gabriele/Desktop/EURAC/MetaRbolomics4Galaxy/mzPeak/small.mzpeak")
        .map_err(|e| extendr_api::Error::from(e.to_string()))?;
    let spec = reader.get_spectrum_by_index(2).unwrap();
    rprintln!("{:?}", spec.description());
    Ok(())
}

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
fn mz_peak_load(filename: String, encryption_key: Option<String>) ->
                extendr_api::Result<Robj> {
    read::load_mzpeak(filename, encryption_key)
}

/// Read an the spectrum of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param index `integer` with the index of the spectrum to extract.
///
/// @noRd
#[extendr]
fn mz_peak_read_spectrum(path: String, index: usize) ->
                         extendr_api::Result<Robj> {
    read_spectrum::read_spectrum(&std::path::PathBuf::from(path), index)
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod MsBackendMzPeak;
    fn mz_peak_reader_test;
    fn mz_peak_load;
    fn mz_peak_read_spectrum;
}
