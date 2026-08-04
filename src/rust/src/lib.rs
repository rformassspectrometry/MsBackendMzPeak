mod parser_fields;
use parser_fields::{
    RSpectrumDescription,
    RSpectrumDescriptionMinimal,
    RFileDescription,
    RSoftware};

mod convert;
use convert::{convert_file, ConvertArgs};

mod sample_metadata;
use sample_metadata::*;

use extendr_api::prelude::*;
use mzdata::{prelude::*};
use mzdata::spectrum::{
    SpectrumDescription
};
use mzdata::mzpeaks::coordinate::{SimpleInterval};
use arrow::array::{Array, AsArray, Float32Array, Float64Array, UInt64Array, RecordBatch};
use mzpeak_prototyping::MzPeakReader;
use std::{
    io,
    panic::{self, AssertUnwindSafe},
    path::{Path, PathBuf},
    fs,
    time::Instant,};
use rayon::prelude::*;
use indexmap::IndexMap;
use arrow_extendr::to::IntoArrowRobj;

/// Load base info of a mzPeak archive(s).
///
/// @param paths `character` Paths to the mzPeak archive(s).
///
/// @return `list` of `data.frame` containing the id of the spectra for each
///     file.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn load_mzpeak(paths: Vec<String>) -> Robj {
    let indexs: Vec<IndexMap<Box<str>, u64>> = paths
        .par_iter()
        .map(|p|
             MzPeakReader::new(p)
                .expect("failed to open mzpeak file")
                .get_index().offsets.clone())
        .collect();

    let r_list: Vec<Robj> = indexs
        .into_iter()
        .map(|idx| {
            let l = idx.len();
            let (names, values): (Vec<String>, Vec<f64>) = idx
                .into_iter()
                .map(|(k, v)| (k.into_string(), v as f64))
                .unzip();

            make_dataframe(
                vec![
                    ("id", Robj::from(names)),
                    ("spectrum_id", values.into_robj())
                ], l)
        })
        .collect();

    List::from_names_and_values(paths, r_list).into_robj()
}


/// Read list of spectrum of a mzPeak archive.
///
/// @param filename `character(1)` Path to the mzPeak archive.
///
/// @param index optional `integer` with the indexs of the Spectrum to extract.
///
/// @return `data.frame` containing the spectra data.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn spectrum_peaks_by_id(path: &str, index: Vec<i32>) -> Robj {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    let index: Vec<usize> = index.into_iter()
        .map(|i| (i - 1) as usize)
        .collect();

    let spectra = reader
        .get_spectra_batch(index.iter().copied())
        .expect("spectrum index out of range");

    // Count total number of peaks across all spectra to size the vectors once.
    let n_total: usize = spectra.iter().map(|s| s.peaks().len()).sum();

    let mut spectrum_id = Vec::with_capacity(n_total);
    let mut mz = Vec::with_capacity(n_total);
    let mut intensity = Vec::with_capacity(n_total);

    for (spec_idx, spectrum) in index.iter().zip(spectra.iter()) {
        let peaks = spectrum.peaks();
        for p in peaks.iter() {
            spectrum_id.push(*spec_idx);
            mz.push(p.mz());
            intensity.push(p.intensity() as f64);
        }
    }

    let peaks_df = make_dataframe(
        vec![
            ("spectrum", Robj::from(spectrum_id)),
            ("mz", Robj::from(mz)),
            ("intensity", Robj::from(intensity))],
        n_total,
    );

    peaks_df
}


/// Read an the peaks of a mzPeak archive(s).
///
/// @param filename `character` Path to the mzPeak archive(s).
///
/// @return `data.frame` containing the spectra data.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn spectrum_peaks(paths: Vec<String>) -> Robj {
    let peaks: Vec<(Vec<i32>, Vec<f64>, Vec<f64>, usize)> = paths
        .par_iter()
        .map(|p| mzpeak_read_all_peaks(p))
        .collect();

    let mut rvec: Vec<Robj> = Vec::with_capacity(peaks.len());
    for p in peaks {
        rvec.push(make_dataframe(
            vec![
                ("spectrum_index", Robj::from(p.0)),
                ("mz", Robj::from(p.1)),
                ("intensity", Robj::from(p.2)),
            ], p.3)
        );
    }

    rvec.into_robj()
}

/// Read spectra data from a single mzPeak archive
///
/// @return tuple with (`spectrum_index`, `mz`, `intensity`, `number of peaks`)
///
/// @noRd
fn mzpeak_read_all_peaks(path: &str) -> (Vec<i32>, Vec<f64>, Vec<f64>, usize) {
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
        // spectrum has no m/z array at all. Treat that as "0 peaks", not an
        // error.
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
    (spectrum_index, mz, intensity, nrow)
}


#[extendr]
fn spectrum_peaks_v2(paths: Vec<String>) -> Robj {
    let peaks: Vec<(Vec<i32>, Vec<f64>, Vec<f32>, usize)> = paths
        .par_iter()
        .map(|p|  mzpeak_read_all_peaks_v2(p))
        .collect();

    let mut rvec: Vec<Robj> = Vec::with_capacity(peaks.len());
    for p in peaks {
        rvec.push(
            make_dataframe(
                vec![
                    ("spectrum_index", Robj::from(p.0)),
                    ("mz", Robj::from(p.1)),
                    ("intensity", Robj::from(p.2)),
                ], p.3)
        );
    }

    rvec.into_robj()
}

fn mzpeak_read_all_peaks_v2(path: &str) -> (Vec<i32>, Vec<f64>, Vec<f32>, usize) {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    let (it, _index) = reader.query_peaks(SimpleInterval::new(0.0, f64::INFINITY), None, None, None).unwrap();
    let mut spectrum_index: Vec<i32> = Vec::new();
    let mut mz: Vec<f64> = Vec::new();
    let mut intensity: Vec<f32> = Vec::new();

    for batch in it.flatten() {
        let root = batch.column(0).as_struct();

        let batch_spectrum_index: &UInt64Array =
            root.column(0).as_any().downcast_ref().unwrap();
        spectrum_index.extend(batch_spectrum_index.values().iter().map(|&v| v as i32));

        let batch_mz: &Float64Array =
            root.column(1).as_any().downcast_ref().unwrap();
        mz.extend(batch_mz.values().iter().copied());

        let batch_intensity: &Float32Array =
            root.column(2).as_any().downcast_ref().unwrap();
        intensity.extend(batch_intensity.values().iter().copied());
    }

    let nrow = mz.len();
    (spectrum_index, mz, intensity, nrow)
}


#[extendr]
fn spectrum_peaks_v3(paths: Vec<String>) -> Robj {
    let peaks: Vec<Robj> = paths
        .iter()
        .map(|p| mzpeak_read_all_peaks_v3(p))
        .collect();

    peaks.into_robj()
}

#[extendr]
fn mzpeak_read_all_peaks_v3(path: &str) -> Robj {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    let (it, _index) = reader.query_peaks(SimpleInterval::new(0.0, f64::INFINITY), None, None, None).unwrap();

    let it_vec: Vec<RecordBatch> = it.flatten().collect();
    it_vec.into_arrow_robj().expect("Error Arrow to R")
}

/// Function to read spectrum metadata fields of a mzPeak file(s).
///
/// @param filename `character` Path to the mzPeak archive(s).
///
/// @param minimal `logical` to get minimal ("id", "index", "ms_level",
///     "polarity") or full metadata. Default: `TRUE`
///
/// @return `list` containing the metadate of the spectra.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn spectrum_metadata(paths: Vec<String>,
                     minimal: bool) -> Robj {
                    // #[extendr(default = true)] minimal: bool) -> Robj {
    let descs: Vec<Vec<SpectrumDescription>> = paths
        .par_iter()
        .map(|p| mzpeak_read_all_spectrum_metadata(p))
        .collect();

    let mut rvec: Vec<Robj> = Vec::with_capacity(descs.len());
    for d in descs {
        rvec.push(descriptions_to_robj(d, minimal));
    }

    rvec.into_robj()
}

/// Read all spectrum metadata of a mzPeak file
fn mzpeak_read_all_spectrum_metadata(path: &str) -> Vec<SpectrumDescription> {
    let mut reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    reader.iter().map(|d| d.description().clone()).collect()
}

/// Builds the R list from the loaded metadata
fn descriptions_to_robj(descriptions: Vec<SpectrumDescription>,
                        minimal: bool) -> Robj {
    if minimal {
        descriptions
            .into_iter()
            .map(|d| RSpectrumDescriptionMinimal(d).into())
            .collect::<Vec<Robj>>()
            .into_robj()
    } else {
        descriptions
            .into_iter()
            .map(|d| RSpectrumDescription(d).into())
            .collect::<Vec<Robj>>()
            .into_robj()
    }
}

#[extendr]
fn spectrum_metadata_v2(paths: Vec<String>,
                    minimal: bool) -> Robj {
                    // #[extendr(default = true)] minimal: bool) -> Robj {
    let descs: Vec<Option<Vec<SpectrumDescription>>> = paths
        .par_iter()
        .map(|p| {
            let mut r = MzPeakReader::new(p)
                .expect("failed to open mzpeak file");
            let meta = r.load_all_spectrum_metadata().unwrap();
            meta.map(|m| m.to_vec())
        }).collect();

    let mut rvec: Vec<Robj> = Vec::with_capacity(descs.len());
    for d in descs {
        match d {
            Some(d) => rvec.push(descriptions_to_robj(d.to_vec(), minimal)),
            None => rvec.push(Robj::from(extendr_api::NULL)),
        }
    }

    rvec.into_robj()
}

/// Function to read sample metadata `FileIndex` of a mzPeak file.
///
/// @param filename Path to the mzPeak archive.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_sample_metadata(path: &str) {
    mzpeak_sample_metadata(path);
}

/// Function to read File Description fields of  metadata `FileIndex` of a
/// mzPeak file.
///
/// @param filename Path to the mzPeak archive.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_file_description(path: &str) -> Robj {
    RFileDescription(mzpeak_file_description(path)).into()
}

/// Function to read File Description fields of  metadata `FileIndex` of a
/// mzPeak file.
///
/// @param filename Path to the mzPeak archive.
///
/// @author Gabriele Tomè
///
/// @noRd
#[extendr]
fn mzpeak_read_software(path: &str) -> Robj {
    mzpeak_softwares(path)
        .iter()
        .map(|sw| Robj::from(RSoftware(sw.clone())))
        .collect::<List>()
        .into_robj()
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

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod MsBackendMzPeak;
    fn load_mzpeak;
    fn spectrum_peaks_by_id;
    fn spectrum_peaks;
    fn spectrum_peaks_v2;
    fn spectrum_peaks_v3;
    fn mzpeak_read_all_peaks_v3;
    fn spectrum_metadata;
    fn spectrum_metadata_v2;
    fn mzpeak_convert;
    fn mzpeak_read_sample_metadata;
    fn mzpeak_read_file_description;
    fn mzpeak_read_software;
}
