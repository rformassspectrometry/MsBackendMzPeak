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
#' - `[`: subset a `MsBackendMzPeak` to the provided spectra (parameter `i`).
#'   Internally, this will subset and re-order the cached data as well as the
#'   indices and mzPeak files. The data in the original mzPeak files is not
#'   affected.
#'
#' @section Data access:
#'
#' Data access functionality.
#'
#' @param drop For `[`: ignored.
#'
#' @param i For `[`: `integer` or `logical` defining to which spectra the data
#'     should be subset.
#'
#' @param j For `[`: ignored.
#'
#' @param object An `MsBackendMzPeak` object.
#'
#' @param x AN `MsBackendMzPeak` object.
#'
#' @param ... additional arguments.
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
        files = "character", # mzPeak file(s)
        sidx = "integer",    # spectrum index
        fidx = "integer"),   # file index for each spectrum index
    prototype = prototype(
        files = character(),
        sidx = integer(),
        fidx = integer(),
        readonly = TRUE, version = "0.1"))

#' @importFrom methods validObject
#'
#' @noRd
setValidity("MsBackendMzPeak", function(object) {
    msg <- NULL
    if (length(object@sidx) != length(object@fidx))
        msg <- paste0("spectrum index does not match file index")
    if (length(object@sidx) != object@nspectra)
        msg <- c(msg, paste0("Number of spectra IDs does not match the number ",
                             "of spectra"))
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
    if (l <- length(object@files)) {
        to <- min(3, l)
        cat("\nfile(s):\n ", paste(basename(object@files[seq_len(to)]),
                                  collapse = "\n "),
            "\n", sep = "")
        if (l > 3)
            cat(" ...", l - 3, "more files\n")
    }
})

#' @rdname MsBackendMzPeak
#'
#' @importFrom methods slot<-
setMethod("[", "MsBackendMzPeak", function(x, i, j, ..., drop = FALSE) {
    x <- callNextMethod()                   # subset cache (parent object)
    slot(x, "sidx", check = FALSE) <- x@sidx[i]
    fidx <- x@fidx[i]
    keep_files <- unique(fidx)
    slot(x, "files", check = FALSE) <- x@files[keep_files]
    slot(x, "fidx", check = FALSE) <- base::match(fidx, keep_files)
    x
})
