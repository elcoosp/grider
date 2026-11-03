# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.3.0] - 2026-11-03

### Added
- `binarize` module: streaming sliding-window integral-image binarization
  (`O(width × height)` time, `O(width × radius)` memory)
- `Binarization::Sauvola` method (Sauvola & Pietikäinen 2000 via integral
  images, Shafait et al. 2008) for degraded documents, with
  `GridConfig::builder().sauvola(k)` and the `--binarization sauvola`
  / `--sauvola-k` CLI flags
- `Grid::try_from_gray_with_config` for direct `GrayImage` input
- `Grid::try_from_binarized` to skip binarization when the input is already
  binary (4.4× faster than the full path)
- `detection::scan_line_flags` / `detection::LineFlags` / `detection::lines_from_flags`:
  fused single-pass row+column emptiness scan
- `docs/BENCHMARKS.md` with before/after criterion numbers

### Changed
- [**breaking**] `GridConfig` gained a `binarization` field (serde
  backward-compatible through `#[serde(default)]`); struct literals must add
  the field or use `..Default::default()`
- Binarization rewritten on a streaming integral ring: 1.6–3.1× faster than
  `imageproc::contrast::adaptive_threshold` with bit-for-bit identical output
  (property-tested), and parallelism is now work-size gated so small images
  never pay rayon coordination overhead
- Row/column emptiness detection replaced by one fused cache-friendly pass
  instead of a row scan plus a strided column scan
- Zero-copy grayscale borrowing: already-gray `DynamicImage` inputs are no
  longer copied before binarization

### Performance
- End-to-end extraction 1.5–3.1× faster across all sizes and patterns
  (2.17× at 1000², 2.55× at 4000², 3.09× at 100²)
- Full numbers and methodology in `docs/BENCHMARKS.md`

## [0.2.0] - 2026-11-01

### Added
- [**breaking**] Modularize the library and overhaul the extraction pipeline


### Continuous Integration
- Full pipeline with coverage gates, cargo-deny and git-cliff releases


### Documentation
- Rewrite README, add CONTRIBUTING, LICENSE and design notes


### Fixed
- Raise MSRV to 1.89


### Tests
- Add integration, property, CLI and snapshot test suites

## [0.1.0] - 2026-10-31

### Added
- Detect some rows & cols- Local threshold- Working line merge- Use rayon- Optional simd- Fix collect_lines- Smallvec & LineInfo- The row and column processing are executed simultaneously, taking advantage of multiple CPU cores.- Make_grid macro- Use try_from- Cell iteration- Add drawing- Take image_path as cli arg- Add filter_smallest_rows, filter_biggest_rows, filter_smallest_columns, filter_biggest_columns- Filter with tolerance


### Continuous Integration
- Setup workflow- Missing clippy, rustfmt- Fix tests & run clippy


### Dependencies
- *(deps)* Update actions/checkout action to v4- *(deps)* Update actions/cache action to v4- *(deps)* Update rust crate criterion to 0.6- *(deps)* Update rust crate criterion to 0.7- *(deps)* Update actions/checkout action to v5- *(deps)* Update rust crate clap to v4.5.50- *(deps)* Update serde monorepo to v1.0.228- *(deps)* Update rust crate thiserror to v2.0.17- *(deps)* Update rust crate proptest to v1.9.0- *(deps)* Update rust crate clap to v4.5.51- *(deps)* Update rust crate image to v0.25.9- *(deps)* Update rust crate clap to v4.5.52- *(deps)* Update rust crate clap to v4.5.53- *(deps)* Update actions/checkout action to v6- *(deps)* Update rust crate insta to v1.44.3- *(deps)* Update tokio-tracing monorepo- *(deps)* Update rust crate criterion to 0.8- *(deps)* Update actions/cache action to v5- *(deps)* Update rust crate tracing to v0.1.44- *(deps)* Update rust crate insta to v1.45.0- *(deps)* Update rust crate thiserror to v2.0.18- *(deps)* Update rust crate insta to v1.46.3- *(deps)* Update rust crate criterion to v0.8.2- *(deps)* Update rust crate clap to v4.5.60- *(deps)* Update rust crate anyhow to v1.0.102- *(deps)* Update rust crate proptest to v1.10.0- *(deps)* Update rust crate image to v0.25.10- *(deps)* Update rust crate clap to v4.6.0- *(deps)* Update rust crate tracing-subscriber to v0.3.23- *(deps)* Update rust crate proptest to v1.11.0- *(deps)* Update rust crate insta to v1.47.0- *(deps)* Update rust crate insta to v1.47.1- *(deps)* Update rust crate insta to v1.47.2- *(deps)* Update rust crate rand to v0.10.1- *(deps)* Update rust crate rayon to v1.12.0- *(deps)* Update rust crate clap to v4.6.1- *(deps)* Update rust crate imageproc to v0.26.2- *(deps)* Update actions/checkout action to v7- *(deps)* Update rust crate anyhow to v1.0.103- *(deps)* Update actions/cache action to v6- *(deps)* Update rust crate smallvec to v1.15.2- *(deps)* Update rust crate insta to v1.48.0- *(deps)* Update rust crate clap to v4.6.2- *(deps)* Update rust crate clap to v4.6.4- *(deps)* Update rust crate thiserror to v2.0.19- *(deps)* Update serde monorepo to v1.0.229- *(deps)* Update rust crate anyhow to v1.0.104- *(deps)* Update rust crate rand to v0.10.2- *(deps)* Update rust crate clap to v4.6.6- *(deps)* Update rust crate thiserror to v2.0.20- *(deps)* Update rust crate smallvec to v1.16.0- *(deps)* Update rust crate smallvec to v1.16.1- *(deps)* Update rust crate clap to v4.6.7- *(deps)* Update rust crate smallvec to v1.16.2- *(deps)* Update rust crate insta to v1.49.0


### Documentation
- Enhance intra-doc- Enhance intra-doc- Fix example comments- Readme- Macros- Enhance drawing- Fix filter


### Fixed
- Adjust SmallVecLine- Use line- LineInfo::new- LineInfo::new everywhere- Debug missing grid var- *(deps)* Update rust crate thiserror to v2.0.11- Main- Clippy all targets- DEFAULT_TOLERANCE- *(deps)* Update serde monorepo to v1.0.218- *(deps)* Update rust crate anyhow to v1.0.96- *(deps)* Update rust crate smallvec to v1.14.0- *(deps)* Update rust crate clap to v4.5.31- *(deps)* Update rust crate insta to v1.42.2- *(deps)* Update rust crate anyhow to v1.0.97- *(deps)* Update rust crate thiserror to v2.0.12- *(deps)* Update serde monorepo to v1.0.219- *(deps)* Update rust crate clap to v4.5.32- *(deps)* Update rust crate clap to v4.5.34- *(deps)* Update rust crate smallvec to v1.15.0- *(deps)* Update rust crate clap to v4.5.35- *(deps)* Update rust crate clap to v4.5.36- *(deps)* Update rust crate anyhow to v1.0.98- *(deps)* Update rust crate clap to v4.5.37- *(deps)* Update rust crate image to v0.25.6- *(deps)* Update rust crate insta to v1.43.1- *(deps)* Update rust crate smallvec to v1.15.1- *(deps)* Update rust crate clap to v4.5.41- *(deps)* Update rust crate proptest to v1.7.0- Use std::hint::black_box- Use unsigned_is_multiple_of- Panic lint fmt- Use unsigned_is_multiple_of in main- Rm [feature(unsigned_is_multiple_of)] no more needed- *(deps)* Update rust crate clap to v4.5.42- *(deps)* Update rust crate anyhow to v1.0.100- *(deps)* Update rust crate image to v0.25.8- *(deps)* Update rust crate insta to v1.43.2- *(deps)* Update rust crate thiserror to v2.0.16- *(deps)* Update rust crate tracing-subscriber to v0.3.20- *(deps)* Update serde monorepo to v1.0.226- *(deps)* Update rust crate proptest to v1.8.0- *(deps)* Update rust crate rayon to v1.11.0- *(deps)* Update rust crate imageproc to 0.26- *(deps)* Update rust crate rand to 0.10.0- *(deps)* Update rust crate imageproc to 0.27


### Miscellaneous
- No print- Log- Clippy- Fmt- Clippy- Split debug in own file- Split filter_lines_with_tolerance- Readme- Up rand dep- Readme- Badge- Badge


### Tests
- Insta snaps- Benches- Add more tests- Improve bench & test- Refactor tests/grid- Enhance test suite- Fix doctest- Add filter_smallest_rows, filter_biggest_rows, filter_smallest_columns

<!-- generated by git-cliff -->
