#' @rdname MsBackendMzPeak
#'
#' @export
MsBackendMzPeak <- function() {
    new("MsBackendMzPeak")
}

#' Internal function to get the metadata (spectraData) from a (single!) mzPeak
#' file.
#'
#' @param x `character(1)` with the path/file name of the mzPeak file
#'
#' @param index `integer` with the index of the spectra within the mzPeak file
#'     from which metadata should be extracted.
#'
#' @param columns `character` with the names of the metadata columns to extract
#'
#' @return `data.frame`, one row per spectrum, with the data.
#'
#' @noRd
.mzpeak_spectra_data <- function(x, index = integer(), columns = character()) {
    ## open connection to the mzPeak file.
    ## read metadata (`columns`) for provided spectra (`index`)
    ## return `data.frame`
    ## example implementations:
    ## https://github.com/rformassspectrometry/Spectra/blob/main/R/MsBackendMzR-functions.R#L23-L49
    ## https://github.com/rformassspectrometry/MsBackendSql/blob/main/R/MsBackendSql-functions.R#L187-L207
}

#' Internal function to get the peaks data (i.e., a `list` of `data.frame` (or
#' matrix?) of m/z and intensity values) from a (single!) mzPeak file
#'
#' @param x `character(1)` with the path/file name from the mzPeak file
#'
#' @param index `integer` with the indices of the spectra from which to read
#'     the peaks data. Reads the full data if not provided.
#'
#' @param columns `character` with the name of the peaks variables to get
#'
#' @return `list` of `data.frame` (or `matrix`?) with the peak data.
#'
#' @noRd
.mzpeak_peak_data <- function(x = character(), index = integer(),
                              columns = c("mz", "intensity")) {
    ## open connection to the mzPeak file
    ## get the data
    ## split into a `list` of `data.frame` (or `matrix`?); `data.frame` would
    ## have faster column access, `matrix` faster row access. We will mostly
    ## access columns, thus maybe use `data.frame`, although the `MsBackend`
    ## specification defines `matrix`...
    ## example implementations:
    ## https://github.com/rformassspectrometry/Spectra/blob/main/R/MsBackendMzR-functions.R#L63-L88
    ## https://github.com/rformassspectrometry/MsBackendSql/blob/main/R/MsBackendSql-functions.R#L92-L119
}

#' Internal function to get the available metadata columns (spectra variables)
#' from an mzPeak file.
#'
#' @param x `character(1)` with the path/file name of the mzPeak file.
#'
#' @return `character` with the available metadata columns.
#'
#' @noRd
.mzpeak_metadata_columns <- function(x) {
}
