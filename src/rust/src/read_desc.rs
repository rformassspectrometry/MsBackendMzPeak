use mzdata::params::{Param, ParamValue};
use mzdata::spectrum::{
    Acquisition, Activation, IsolationWindow, Precursor, ScanEvent, ScanWindow,
    SelectedIon, SpectrumDescription,
};
use extendr_api::prelude::*;
/// RParam
struct RParam<'a>(&'a Param);

impl<'a> From<RParam<'a>> for Robj {
    fn from(val: RParam<'a>) -> Self {
        let p = val.0;
        let value_robj: Robj = if let Ok(f) = p.to_f64() { f.into_robj()
            } else if let Ok(i) = p.to_i64() { i.into_robj()
            } else { p.as_str().as_ref().into_robj() };
        List::from_names_and_values(
            &["name", "value", "accession", "controlled_vocabulary", "unit"],
            &[
                p.name.as_str().into_robj(),
                value_robj,
                p.accession.map(|a| a.to_string()).as_deref().into_robj(),
                format!("{:?}", p.controlled_vocabulary).into_robj(),
                format!("{:?}", p.unit).into_robj()
            ],
        ).unwrap().into_robj()
    }
}

fn params_to_robj(params: &[Param]) -> Robj {
    params.iter().map(|p| Robj::from(RParam(p))).collect::<List>().into_robj()
}

/// RScanWindow
struct RScanWindow<'a>(&'a ScanWindow);

impl<'a> From<RScanWindow<'a>> for Robj {
    fn from(val: RScanWindow<'a>) -> Self {
        let w = val.0;
        List::from_names_and_values(
            &["lower_bound", "upper_bound"],
            &[w.lower_bound.into_robj(), w.upper_bound.into_robj()],
        ).unwrap().into_robj()
    }
}

/// RScanEvent
struct RScanEvent<'a>(&'a ScanEvent);

impl<'a> From<RScanEvent<'a>> for Robj {
    fn from(val: RScanEvent<'a>) -> Self {
        let s = val.0;
        let windows: List = s.scan_windows.iter()
                                .map(|w| Robj::from(RScanWindow(w))).collect();
        let params_robj = match &s.params {
            Some(ps) => params_to_robj(ps),
            None => r!(NULL),
        };
        List::from_names_and_values(
            &["start_time", "injection_time", "scan_windows",
              "instrument_configuration_id", "params"],
            &[
                s.start_time.into_robj(),
                s.injection_time.into_robj(),
                windows.into_robj(),
                s.instrument_configuration_id.into_robj(),
                params_robj
            ],
        ).unwrap().into_robj()
    }
}

/// RAcquisition
struct RAcquisition<'a>(&'a Acquisition);

impl<'a> From<RAcquisition<'a>> for Robj {
    fn from(val: RAcquisition<'a>) -> Self {
        let a = val.0;
        let scans: List = a.scans.iter().map(|s| Robj::from(RScanEvent(s)))
                            .collect();
        let params_robj = match &a.params {
            Some(ps) => params_to_robj(ps),
            None => r!(NULL),
        };
        List::from_names_and_values(
            &["scans", "combination", "params"],
            &[
                scans.into_robj(),
                format!("{:?}", a.combination).into_robj(),
                params_robj
            ],
        ).unwrap().into_robj()
    }
}

/// RSelectedIon
struct RSelectedIon<'a>(&'a SelectedIon);

impl<'a> From<RSelectedIon<'a>> for Robj {
    fn from(val: RSelectedIon<'a>) -> Self {
        let ion = val.0;
        let params_robj = match &ion.params {
            Some(ps) => params_to_robj(ps),
            None => r!(NULL),
        };
        List::from_names_and_values(
            &["mz", "intensity", "charge", "params"],
            &[
                ion.mz.into_robj(),
                ion.intensity.into_robj(),
                ion.charge.into_robj(),
                params_robj
            ],
        ).unwrap().into_robj()
    }
}

/// RIsolationWindow
struct RIsolationWindow<'a>(&'a IsolationWindow);

impl<'a> From<RIsolationWindow<'a>> for Robj {
    fn from(val: RIsolationWindow<'a>) -> Self {
        let w = val.0;
        List::from_names_and_values(
            &["target", "lower_bound", "upper_bound", "flags"],
            &[
                w.target.into_robj(),
                w.lower_bound.into_robj(),
                w.upper_bound.into_robj(),
                format!("{:?}", w.flags).into_robj()
            ],
        ).unwrap().into_robj()
    }
}

/// RActivation
struct RActivation<'a>(&'a Activation);

impl<'a> From<RActivation<'a>> for Robj {
    fn from(val: RActivation<'a>) -> Self {
        let act = val.0;
        let methods: List = act.methods().into_iter()
                .map(|m| format!("{:?}", m).into_robj()).collect();
        let params_robj = params_to_robj(&act.params);
        List::from_names_and_values(
            &["methods", "energy", "params"],
            &[
                methods.into_robj(),
                act.energy.into_robj(),
                params_robj
            ],
        ).unwrap().into_robj()
    }
}

/// RPrecursor
struct RPrecursor<'a>(&'a Precursor);

impl<'a> From<RPrecursor<'a>> for Robj {
    fn from(val: RPrecursor<'a>) -> Self {
        let prec = val.0;
        let ions: List = prec.ions.iter().map(|i| Robj::from(RSelectedIon(i)))
                            .collect();
        List::from_names_and_values(
            &["ions", "isolation_window", "precursor_id", "product_id",
              "activation"],
            &[
                ions.into_robj(),
                RIsolationWindow(&prec.isolation_window).into(),
                prec.precursor_id.as_deref().into_robj(),
                prec.product_id.as_deref().into_robj(),
                RActivation(&prec.activation).into()
            ],
        ).unwrap().into_robj()
    }
}

/// RSpectrumDescription
pub struct RSpectrumDescription(pub SpectrumDescription);

impl From<RSpectrumDescription> for Robj {
    fn from(val: RSpectrumDescription) -> Self {
        let desc = &val.0;
        let params_robj = params_to_robj(&desc.params);
        let precursors: List = desc.precursor.iter()
            .map(|p| Robj::from(RPrecursor(p))).collect();
        List::from_names_and_values(
            &["id", "index", "ms_level", "polarity", "signal_continuity",
              "params", "acquisition", "precursor"],
            &[
                desc.id.as_str().into_robj(),
                desc.index.into_robj(),
                desc.ms_level.into_robj(),
                format!("{:?}", desc.polarity).into_robj(),
                format!("{:?}", desc.signal_continuity).into_robj(),
                params_robj,
                RAcquisition(&desc.acquisition).into(),
                precursors.into_robj()
            ]
        ).unwrap().into_robj()
    }
}
