
convertToMzPeak <- function(path, pattern = ".mzML$|.mzml", output_dir = "./") {
    list_files <- list.files(path, pattern = pattern, full.names = TRUE)
    if (length(list_files))
        stop("No files matching the pattern")

    for (file in list_files) {
        tryCatch({
            res_file <- file.path(output_dir, paste0(basename(file), ".mzPeak"))
            mzpeak_convert(list_files, res_file)
        }, error = function(e){
            stop("Failed to convert ", file, ".\n", e)
        })
    }
}
