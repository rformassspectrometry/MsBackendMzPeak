#' @importFrom rextendr rust_sitrep
.onLoad <- function(libname, pkgname) {
    rust_sitrep()
}
