use mzdata::meta::*;
use mzpeak_prototyping::MzPeakReader;
use std::collections::HashMap;

pub fn mzpeak_sample_metadata(path: &str) {
    let reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");

    let file_index = reader.file_index();
    println!(" {:?}", file_index)
}

pub fn mzpeak_file_description(path: &str) -> FileDescription {
    let reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");
    reader.file_description().clone()
}

pub fn mzpeak_instrument(path: &str) -> HashMap<u32, InstrumentConfiguration> {
    let reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");
    reader.instrument_configurations().clone()
}

pub fn mzpeak_softwares(path: &str) -> Vec<Software> {
    let reader = MzPeakReader::new(path)
        .expect("failed to open mzpeak file");
    reader.softwares().clone()
}
