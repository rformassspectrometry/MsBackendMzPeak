#' Convert mzML Files to mzPeak Format
#'
#' Converts one or more mzML files found in a directory into the mzPeak format.
#' This is a batch wrapper that identifies matching files by a regular
#' expression pattern, then converts each one individually.
#'
#' @details
#' The actual conversion is performed by the internal *Rust* function
#' `mzpeak_convert()`, exposed to R via *extendr*. For each input file,
#' `convertToMzPeak()` builds an output path (same basename, `.mzPeak`
#' extension, placed in `output_dir`) and passes both the input and output
#' paths to `mzpeak_convert()`, which:
#' - Resolves the input filename and output path as native Rust `Path` \
#'   `PathBuf` objects.
#' - Performs the conversion using the [mzPeak example code](https://github.com/HUPO-PSI/mzPeak/blob/main/examples/convert.rs)
#'   with default parameter.
#'
#' Because the conversion logic lives in compiled Rust code, any low-level I/O
#' or parsing errors are propagated back to R as an `io::Result` failure, which
#' is caught in `convertToMzPeak()` via `tryCatch` and re-thrown as an R error
#' with a message identifying the offending file.
#'
#' @param path `character(1)` Path to the directory containing the files to
#'     convert.
#'
#' @param pattern `character(1)` Regular expression used to match files within
#'     `path`. Defaults to `".mzML$|.mzml$"`.
#'
#' @param output_dir `character(1)` Path to the directory where the converted
#'     `mzPeak` files will be saved. Defaults to the current directory.
#'
#' @importFrom tools file_path_sans_ext
#'
#' @author Gabriele Tomè, Joshua Klein
#'
#' @export
convertToMzPeak <- function(path, pattern = ".mzML$|.mzml$",
                            output_dir = "./") {
    list_files <- list.files(path, pattern = pattern, full.names = TRUE)

    if (!length(list_files))
        stop("No files matching the pattern")

    if (!dir.exists(output_dir))
        dir.create(output_dir, showWarnings = FALSE, recursive = TRUE)

    for (file in list_files) {
        tryCatch({
            res_file <- file.path(output_dir,
                                    paste0(file_path_sans_ext(basename(file)),
                                    ".mzPeak"))
            mzpeak_convert(file, res_file)
        }, error = function(e){
            stop("Failed to convert ", file, ".\n", e)
        })
    }
}

