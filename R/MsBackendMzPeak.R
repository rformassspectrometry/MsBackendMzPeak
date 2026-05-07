## mzpeak uses R6 because it does not have copy on write!
## $spectra are Arrow *things*? can filter them on disk before materializing
## them?
## mzpeak has a C extension to allow reading directly from zip archive (?)

#' @title `Spectra` MS backend to read data from files in mzPeak format
#'
#' @aliases MsBackendMzPeak-class
#'
#' @description
#'
#' The `MsBackendMzPeak` is an implementation of the [Spectra::MsBackend()]
#' class for [Spectra::Spectra()] objects adding support for MS data import
#' from the [HUPO-PSI mzPeak](https://github.com/HUPO-PSI/mzPeak) file format.
#' `MsBackendMzPeak` is an *on-disk* implementation keeping only minimal
#' information in memory while loading any data (peaks data or spectra metadata)
#' from the original mzPeak files on-the-fly.
#'
#' @details
#'
#' The `MsBackendMzPeak` backend uses functionality from the
#' [*mzpeak*](https://github.com/HUPO-PSI/mzpeak/R/mzpeak) R package for
#' accessing data in mzPeak files.
#'
#' The `MsBackendMzPeak` extends the [Spectra::MsBackendCached()]` backend
#' allowing thus to intermediately store spectrum metadata (*spectra variables*)
#' in a `data.frame` within the object (i.e., **in memory**).
#'
#' @section Creation of backend objects:
#'
#' New `MsBackendMzPeak` backend objects can be created with the
#' `MsBackendMzPeak()` function.
#'
#' @section Subset, merge and filter data:
#'
#' Subset...
#'
#' @section Data access:
#'
#' Data access functionality.
#'
#' @param object An `MsBackendMzPeak` object.
#'
#' @author Johannes Rainer, Gabriele Tomè
#'
#' @exportClass MsBackendMzPeak
#'
#' @name MsBackendMzPeak
NULL

#' @importClassesFrom S4Vectors DataFrame
#'
#' @importClassesFrom Spectra MsBackendCached
setClass(
    "MsBackendMzPeak",
    contains = "MsBackendCached",
    slots = c(
        mzPeakFile = "character",
        spectraIds = "integer"),
    prototype = prototype(
        mzPeakFile = character(),
        spectraIds = integer(),
        readonly = TRUE, version = "0.1"))

#' @importFrom methods validObject
#'
#' @noRd
setValidity("MsBackendMzPeak", function(object) {
    msg <- NULL
    if (length(object@spectraIds) != object@nspectra)
        msg <- paste0("Number of spectra IDs does not match the number ",
                      "of spectra")
    if (is.null(msg)) TRUE
    else msg
})

#' @importMethodsFrom Spectra show
#'
#' @importFrom methods callNextMethod new
#'
#' @exportMethod show
#'
#' @rdname MsBackendMzPeak
setMethod("show", "MsBackendMzPeak", function(object) {
    callNextMethod()
    if (l <- length(object@mzPeakFile)) {
        to <- min(3, l)
        cat("\nfile(s):\n ", paste(basename(object@mzPeakFile[seq_len(to)]),
                                  collapse = "\n "),
            "\n", sep = "")
        if (l > 3)
            cat(" ...", l - 3, "more files\n")
    }
})
