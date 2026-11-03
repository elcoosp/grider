# Benchmarks

All numbers below were measured with the criterion suites in `benches/`
(`cargo bench --features test-utils`) on the same machine (2 vCPU x86-64
Linux, `sample_size = 20`), comparing the 0.2 pipeline (`--save-baseline
before`) against the 0.3 pipeline (`--save-baseline after`). Every change is
statistically significant (`p = 0.00 < 0.05`); the median ratio is reported.

## End-to-end extraction (`Grid::try_from_image_with_config`)

### By image size (Checkerboard, default config)

| Size | 0.2 | 0.3 | Change | Speed-up |
| :--- | ---: | ---: | ---: | ---: |
| 100 × 100 | 84.7 µs | 28.5 µs | −67.6 % | **3.09×** |
| 500 × 500 | 1.052 ms | 594 µs | −46.6 % | **1.87×** |
| 1000 × 1000 | 3.800 ms | 1.744 ms | −53.9 % | **2.17×** |
| 2000 × 2000 | 14.96 ms | 6.855 ms | −55.0 % | **2.22×** |
| 4000 × 4000 | 66.38 ms | 26.10 ms | −60.8 % | **2.55×** |

### By pattern (1000 × 1000, default config)

| Pattern | 0.2 | 0.3 | Speed-up |
| :--- | ---: | ---: | ---: |
| Checkerboard | 3.783 ms | 1.760 ms | **2.15×** |
| HorizontalStripes | 3.880 ms | 1.772 ms | **2.24×** |
| VerticalStripes | 4.025 ms | 1.769 ms | **2.29×** |
| Sparse | 4.667 ms | 1.697 ms | **2.76×** |
| Dense | 3.792 ms | 1.702 ms | **2.22×** |

### By configuration (1000 × 1000 Checkerboard)

| Configuration | 0.2 | 0.3 | Speed-up |
| :--- | ---: | ---: | ---: |
| small block, sequential (r = 4) | 3.697 ms | 2.324 ms | **1.60×** |
| small block, parallel (r = 4) | 3.730 ms | 1.706 ms | **2.23×** |
| large block, sequential (r = 24) | 3.741 ms | 2.363 ms | **1.58×** |
| large block, parallel (r = 24) | 3.777 ms | 1.798 ms | **2.07×** |

### Threshold block size sweep (1000 × 1000 Dense)

| Block radius | 0.2 | 0.3 | Speed-up |
| ---: | ---: | ---: | ---: |
| 4 | 3.741 ms | 1.667 ms | **2.24×** |
| 8 | 3.727 ms | 1.711 ms | **2.19×** |
| 12 | 3.697 ms | 1.722 ms | **2.15×** |
| 16 | 3.711 ms | 1.716 ms | **2.15×** |
| 24 | 3.995 ms | 1.776 ms | **2.17×** |
| 32 | 3.812 ms | 1.746 ms | **2.20×** |

The new pipeline is `O(width × height)` in the *total* pixel count and
independent of the block radius, matching the old behaviour but with a much
smaller constant.

## Stage benchmarks (`benches/pipeline.rs`)

### Binarization: streaming integral vs. `imageproc` reference

Same semantics, same output (verified bit-for-bit by property tests).

| Image | `imageproc::adaptive_threshold` | `grider` sequential | `grider` parallel | `grider` Sauvola |
| :--- | ---: | ---: | ---: | ---: |
| 1000 × 1000 | 3.765 ms | 1.912 ms (**1.97×**) | 1.429 ms (**2.63×**) | 3.777 ms |
| 4000 × 4000 | 65.36 ms | 30.90 ms (**2.12×**) | 20.81 ms (**3.14×**) | 61.94 ms |

Sauvola costs about the same as the old adaptive mean while providing the
state-of-the-art local threshold for degraded documents — the adaptive mean
itself became 2–3× faster in the same release.

### Fused emptiness scan (`scan_line_flags`)

The single cache-friendly pass replaced a row scan plus a cache-hostile
strided column scan:

| Image | Sequential | Parallel |
| :--- | ---: | ---: |
| 1000 × 1000 | 382 µs (~0.38 ns/px) | 379 µs |
| 4000 × 4000 | 6.52 ms (~0.41 ns/px) | 5.08 ms |

On this 2-vCPU machine the scan is bandwidth bound, so parallelism pays only
at very large sizes; the built-in work-size heuristics fall back to the
sequential path when rayon coordination would cost more than it saves (see
`parallel_worthwhile` in `src/binarize.rs`).

### Entry points (1000 × 1000 Dense, default config)

| Entry point | Time | Notes |
| :--- | ---: | :--- |
| `Grid::try_from_image_with_config` (Luma8) | 1.701 ms | zero-copy gray borrow |
| `Grid::try_from_image_with_config` (RGB) | 4.982 ms | + color conversion (image crate) |
| `Grid::try_from_gray_with_config` | 1.721 ms | no dynamic-image wrapper |
| `Grid::try_from_binarized` | 386 µs | skips binarization: **4.4× vs. full path** |

## Memory

The adaptive mean binarizer no longer materializes a whole integral image:

| Image | 0.2 integral image (`u32`) | 0.3 streaming ring (`f64`) |
| :--- | ---: | ---: |
| 1000 × 1000, r = 12 | ~4 MB | ~104 KB |
| 4000 × 4000, r = 12 | ~64 MB | ~1.7 MB |
| 4000 × 4000, r = 64 | ~64 MB | ~8.3 MB |

Peak extra memory is `O(width × radius)` (two bands at most on this machine)
instead of `O(width × height)`.

## Reproducing

```bash
cargo bench --features test-utils -- --save-baseline before   # on 0.2
# apply the 0.3 changes
cargo bench --features test-utils -- --save-baseline after
cargo bench --features test-utils -- --baseline before        # comparison
```
