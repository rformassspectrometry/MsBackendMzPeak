mod read;
mod read_spectrum;
mod read_desc;

use extendr_api::prelude::*;

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
fn mzpeak_read_desc(filename: String, encryption_key: Option<String>,
                    #[default = "NULL"] index: Option<usize>) ->
                    extendr_api::Result<Robj> {
    let n_spectra = read::mzpeak_n_spectra(filename.clone(),
                                            encryption_key).unwrap();
    let descriptions = match &index {
        Some(i) => {
            if *i >= n_spectra {
                return Err(extendr_api::Error::Other(format!("No spectrum available for index {i}")));
            }
            vec![read_desc::read_desc(filename.clone(), *i)?]
        },
        None => {
            let mut descriptions = Vec::with_capacity(n_spectra);
            for i in 0..n_spectra {
                descriptions.push(read_desc::read_desc(filename.clone(), i)?);
            }
            descriptions
        },
    };
    Ok(List::from_values(descriptions).into_robj())
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
fn mzpeak_read_spectrum(filename: String, encryption_key: Option<String>,
                        #[default = "NULL"] index: Option<usize>) ->
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

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod MsBackendMzPeak;
    fn mzpeak_info;
    fn mzpeak_read_desc;
    fn mzpeak_read_spectrum;
}
