test_that("MsBackendMzPeak class works", {
    a <- new("MsBackendMzPeak")
    expect_true(validObject(a))
    a@spectraIds <- 1:10
    expect_error(validObject(a), "Number of spectra IDs")
    a@spectraIds <- integer()
    res <- capture.output(show(a))
    expect_match(res, "MsBackendMzPeak")

    a@mzPeakFile <- c("a", "b", "c", "d")
    res <- capture.output(show(a))
    expect_match(res[length(res)], "more file")
})
