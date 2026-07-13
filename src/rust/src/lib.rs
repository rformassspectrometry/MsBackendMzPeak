mod read_desc;
use read_desc::RSpectrumDescription;

mod convert;
use convert::{convert_file, ConvertArgs};

use extendr_api::prelude::*;
use mzdata::{prelude::*};
use mzpeak_prototyping::MzPeakReader;
use std::{
    io,
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    fs,
    time::Instant,};

/// Build a data.frame without going through R's `data.frame()` constructor
///
/// @author Gabriele Tomè
///
/// @noRd
fn make_dataframe(columns: Vec<(&str, Robj)>, nrow: usize) -> Robj {
    let names: Vec<&str> = columns.iter().map(|(n, _)| *n).collect();
    let values: Vec<Robj> = columns.into_iter().map(|(_, v)| v).collect();

    let mut df: Robj = List::from_values(values).into();
    df.set_attrib(names_symbol(), Robj::from(names)).unwrap();
    df.set_attrib(class_symbol(),
                 Robj::from("data.frame"))
        .unwrap();
    df.set_attrib(
        row_names_symbol(),
        (1..=nrow as i32).collect_robj(),
    )
    .unwrap();
    df
}

/// Read an the spectrum of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param index optional `integer` with the index of the Spectrum to extract.
///     If not provided, the function extract all the Spectrum.
///
/// @return `list` containing the spectra data.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_peaks(path: &str, index: i32) -> List {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");
    let spectrum = reader
        .get_spectrum_by_index(index as usize)
        .expect("spectrum index out of range");

    let desc = spectrum.description();
    let peaks = spectrum.peaks();
    let n = peaks.len();

    let mut mz = Vec::with_capacity(n);
    let mut intensity = Vec::with_capacity(n);
    for p in peaks.iter() {
        mz.push(p.mz());
        intensity.push(p.intensity() as f64);
    }

    let peaks_df = make_dataframe(
        vec![("mz", Robj::from(mz)), ("intensity", Robj::from(intensity))],
        n,
    );

    list!(
        id = desc.id.clone(),
        ms_level = desc.ms_level as i32,
        polarity = format!("{:?}", desc.polarity),
        n_peaks = n as i32,
        peaks = peaks_df
    )
}


/// Read an the peaks of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @return `data.frame` containing the spectra data.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_all_peaks(path: &str) -> Robj {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    let mut spectrum_index: Vec<i32> = Vec::new();
    let mut mz: Vec<f64> = Vec::new();
    let mut intensity: Vec<f64> = Vec::new();
    let mut n_empty = 0usize;
    let mut n_total = 0usize;

    // Suppress the default Rust panic printout to stderr — we handle and
    // report failures ourselves below, so the raw panic message is just noise.
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    for (i, spectrum) in (&mut reader).enumerate() {
        n_total += 1;

        // spectrum.peaks() panics internally (NotFound(MZArray)) when a
        // spectrum has no m/z array at all — typically an MS2 scan with
        // zero detected fragment ions. Treat that as "0 peaks", not an error.
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let peaks = spectrum.peaks();
            let m: Vec<f64> = peaks.iter().map(|p| p.mz()).collect();
            let it: Vec<f64> = peaks.iter().map(|p| p.intensity() as f64).collect();
            (m, it)
        }));

        match result {
            Ok((m, it)) => {
                let n = m.len();
                spectrum_index.reserve(n);
                mz.reserve(n);
                intensity.reserve(n);

                spectrum_index.extend(std::iter::repeat(i as i32).take(n));
                mz.extend(m);
                intensity.extend(it);
            }
            Err(_) => {
                n_empty += 1;
            }
        }
    }

    panic::set_hook(prev_hook);

    if n_empty > 0 {
        eprintln!(
            "mzpeak: {n_empty} of {n_total} spectra had no readable peak array (treated as 0 peaks)"
        );
    }

    let nrow = mz.len();
    make_dataframe(
        vec![
            ("spectrum_index", Robj::from(spectrum_index)),
            ("mz", Robj::from(mz)),
            ("intensity", Robj::from(intensity)),
        ],
        nrow,
    )
}

/// Function to read description field of a mzPeak file.
///
/// @param filename Path to the mzPeak archive.
///
/// @return `list` containing the matadate of the spectra.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_all_metadata(path: &str) -> Robj {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    let mut list_desc: Vec<RSpectrumDescription> = Vec::new();

    for spectrum in &mut reader {
        let desc = spectrum.description();
        list_desc.push(RSpectrumDescription(desc.clone()).into());
    }
    let list: List = list_desc
        .into_iter()
        .map(Robj::from)   // uses your `From<RSpectrumDescription> for Robj` impl
        .collect();
    list.into_robj()
}

/// Function to convert files to mzPeak
///
/// @param filename `character(1)` Path to the file to convert.
///
/// @param outfile `character(1)` Path where save the mzPeak archive.
///
/// @author Gabriele Tomè, Joshua Klein
///
/// @noRd
#[extendr]
fn mzpeak_convert(filename: String, outfile: String) -> io::Result<()>{
    let filename = Path::new(&filename);
    let outfile = PathBuf::from(outfile);

    let start = Instant::now();
    // TODO: improve and handle the different parameters
    let args = ConvertArgs::default();

    let stat = fs::metadata(&filename)?;
    let size = stat.len() as f64 / 1e9;
    println!("{} is {size:0.3}GB", filename.display());

    convert_file(filename, &outfile, &args)?;

    println!("{:0.2} seconds elapsed", start.elapsed().as_secs_f64());
    let stat = fs::metadata(&outfile)?;
    let size = stat.len() as f64 / 1e9;
    println!("{} is now {size:0.3}GB\n", outfile.display());

    Ok(())
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod MsBackendMzPeak;
    fn mzpeak_read_peaks;
    fn mzpeak_read_all_peaks;
    fn mzpeak_read_all_metadata;
    fn mzpeak_convert;
}
