test_that("MsBackendMzPeak works", {
    a <- MsBackendMzPeak()
    expect_s4_class(a, "MsBackendMzPeak")
})
