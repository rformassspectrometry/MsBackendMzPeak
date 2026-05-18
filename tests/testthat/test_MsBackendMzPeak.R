test_that("MsBackendMzPeak class works", {
    a <- new("MsBackendMzPeak")
    expect_true(validObject(a))
    a@sidx <- 1:10
    expect_error(validObject(a), "spectrum index does not match file index")
    expect_error(validObject(a), "Number of spectra IDs")
    a@fidx <- rep(1L, 10)
    expect_error(validObject(a), "Number of spectra IDs")
    a <- MsBackendMzPeak()
    res <- capture.output(show(a))
    expect_match(res, "MsBackendMzPeak")

    a@files <- c("a", "b", "c", "d")
    res <- capture.output(show(a))
    expect_match(res[length(res)], "more file")
})

test_that("[,MsBackendMzPeak works", {
    be <- backendInitialize(MsBackendCached(), nspectra = 10)
    be <- as(be, "MsBackendMzPeak")
    expect_error(validObject(be), "Number of spectra IDs")
    be@files <- c("a", "b", "c", "d")
    be@fidx <- c(1L, 1L, 2L, 2L, 3L, 3L, 3L, 4L, 4L, 4L)
    be@sidx <- c(1L, 2L, 1L, 2L, 1L, 2L, 3L, 1L, 2L, 3L)
    expect_no_error(validObject(be))

    res <- be[c(3, 9, 4, 10)]
    expect_no_error(validObject(res))
    expect_equal(res@sidx, c(1L, 2L, 2L, 3L))
    expect_equal(res@fidx, c(1L, 2L, 1L, 2L))
    expect_equal(res@files, c("b", "d"))

    ## with duplicated spectra
    res <- be[c(4, 5, 4, 10, 1)]
    expect_no_error(validObject(res))
    expect_equal(res@files, c("b", "c", "d", "a"))
    expect_equal(res@fidx, c(1L, 2L, 1L, 3L, 4L))
    expect_equal(res@sidx, c(2L, 1L, 2L, 3L, 1L))
})
