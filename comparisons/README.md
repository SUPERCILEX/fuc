# Benchmarks

The benchmarks take several hours to run and create hundreds of millions of files. Run at your own
risk. :)

## Setup

Run all commands in the root of this repository.

```bash
cargo install hyperfine ftzz
cargo b --workspace --release
mkdir benches
```

## Remove

### Run

```bash
for num_bytes in {0,100M}; do for num_files in {10,10_000,100_000,1M}; do hyperfine --warmup 3 -N --export-markdown "benches/remove_${num_files}_files_${num_bytes}_bytes.md" --export-json "benches/remove_${num_files}_files_${num_bytes}_bytes.json" --prepare "ftzz g -n ${num_files} -b ${num_bytes} /tmp/ftzz" "rm -r /tmp/ftzz" "./target/release/stdlib_rm /tmp/ftzz" "./target/release/rayon_rm /tmp/ftzz" "./target/release/rmz /tmp/ftzz"; done; done
for num_bytes in {0,100M}; do hyperfine --warmup 3 -N --export-markdown "benches/remove_100_000_files_${num_bytes}_bytes_0_depth.md" --export-json "benches/remove_100_000_files_${num_bytes}_bytes_0_depth.json" --prepare "ftzz g -n 100_000 -b ${num_bytes} -d 0 /tmp/ftzz" "rm -r /tmp/ftzz" "./target/release/stdlib_rm /tmp/ftzz" "./target/release/rayon_rm /tmp/ftzz" "./target/release/rmz /tmp/ftzz"; done
```

### Results

#### `10_files_0_bytes.md`

| Command                                | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------------------------------|----------:|---------:|---------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 1.1 ± 0.1 |      1.0 |      2.2 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  | 1.5 ± 0.3 |      1.2 |      3.1 | 1.36 ± 0.26 |
| `rm -r /tmp/ftzz`                      | 1.8 ± 0.1 |      1.6 |      2.2 | 1.60 ± 0.11 |
| `./target/release/stdlib_rm /tmp/ftzz` | 2.2 ± 0.1 |      2.1 |      2.8 | 1.97 ± 0.14 |

#### `10_000_files_0_bytes.md`

| Command                                |   Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------------------------------|------------:|---------:|---------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  45.0 ± 1.8 |     42.5 |     48.8 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  54.7 ± 2.1 |     51.7 |     59.9 | 1.22 ± 0.07 |
| `./target/release/stdlib_rm /tmp/ftzz` | 106.6 ± 1.1 |    105.3 |    109.5 | 2.37 ± 0.10 |
| `rm -r /tmp/ftzz`                      | 118.5 ± 1.2 |    116.4 |    121.0 | 2.64 ± 0.11 |

#### `100_000_files_0_bytes.md`

| Command                                |     Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------------------------------|--------------:|---------:|---------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 486.4 ± 105.8 |    350.1 |    645.5 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  678.3 ± 95.9 |    488.8 |    812.5 | 1.39 ± 0.36 |
| `./target/release/stdlib_rm /tmp/ftzz` |  838.7 ± 21.7 |    816.4 |    885.7 | 1.72 ± 0.38 |
| `rm -r /tmp/ftzz`                      |  856.7 ± 29.3 |    801.5 |    906.3 | 1.76 ± 0.39 |

#### `1M_files_0_bytes.md`

| Command                                |       Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------------------------|---------------:|--------:|--------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  5.024 ± 0.094 |   4.931 |   5.193 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  6.422 ± 0.270 |   6.121 |   6.995 | 1.28 ± 0.06 |
| `./target/release/stdlib_rm /tmp/ftzz` | 11.753 ± 0.106 |  11.599 |  11.876 | 2.34 ± 0.05 |
| `rm -r /tmp/ftzz`                      | 11.926 ± 0.097 |  11.822 |  12.071 | 2.37 ± 0.05 |

#### `10_files_100M_bytes.md`

| Command                                |  Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------------------------------|-----------:|---------:|---------:|------------:|
| `./target/release/rayon_rm /tmp/ftzz`  |  8.1 ± 0.2 |      7.6 |      8.6 |        1.00 |
| `./target/release/rmz /tmp/ftzz`       |  9.0 ± 0.3 |      8.3 |      9.6 | 1.11 ± 0.05 |
| `./target/release/stdlib_rm /tmp/ftzz` | 25.5 ± 2.7 |     24.7 |     44.6 | 3.13 ± 0.34 |
| `rm -r /tmp/ftzz`                      | 25.9 ± 0.7 |     25.1 |     27.6 | 3.18 ± 0.12 |

#### `10_000_files_100M_bytes.md`

| Command                                |   Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------------------------------|------------:|---------:|---------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  53.0 ± 1.6 |     49.6 |     55.9 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  63.4 ± 1.7 |     61.2 |     66.5 | 1.20 ± 0.05 |
| `./target/release/stdlib_rm /tmp/ftzz` | 155.6 ± 1.3 |    154.0 |    158.9 | 2.94 ± 0.09 |
| `rm -r /tmp/ftzz`                      | 168.2 ± 1.5 |    166.6 |    172.4 | 3.18 ± 0.10 |

#### `100_000_files_100M_bytes.md`

| Command                                |      Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------------------------|--------------:|--------:|--------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 0.389 ± 0.029 |   0.361 |   0.462 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  | 0.550 ± 0.088 |   0.453 |   0.694 | 1.41 ± 0.25 |
| `./target/release/stdlib_rm /tmp/ftzz` | 1.026 ± 0.015 |   1.012 |   1.067 | 2.64 ± 0.20 |
| `rm -r /tmp/ftzz`                      | 1.044 ± 0.008 |   1.031 |   1.055 | 2.69 ± 0.20 |

#### `1M_files_100M_bytes.md`

| Command                                |       Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------------------------|---------------:|--------:|--------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  7.539 ± 0.167 |   7.360 |   7.828 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  9.052 ± 0.334 |   8.601 |   9.875 | 1.20 ± 0.05 |
| `./target/release/stdlib_rm /tmp/ftzz` | 28.069 ± 2.587 |  25.546 |  34.910 | 3.72 ± 0.35 |
| `rm -r /tmp/ftzz`                      | 27.424 ± 1.179 |  25.732 |  28.971 | 3.64 ± 0.18 |

#### `100_000_files_0_bytes_0_depth.md`

| Command                                |    Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:---------------------------------------|-------------:|---------:|---------:|------------:|
| `./target/release/stdlib_rm /tmp/ftzz` |  938.8 ± 6.9 |    930.4 |    948.9 |        1.00 |
| `./target/release/rmz /tmp/ftzz`       |  939.5 ± 6.8 |    929.8 |    950.1 | 1.00 ± 0.01 |
| `rm -r /tmp/ftzz`                      |  972.2 ± 2.7 |    968.5 |    977.9 | 1.04 ± 0.01 |
| `./target/release/rayon_rm /tmp/ftzz`  | 1254.0 ± 8.7 |   1240.7 |   1265.5 | 1.34 ± 0.01 |

#### `100_000_files_100M_bytes_0_depth.md`

| Command                                |      Mean [s] | Min [s] | Max [s] |    Relative |
|:---------------------------------------|--------------:|--------:|--------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 1.156 ± 0.007 |   1.143 |   1.166 |        1.00 |
| `./target/release/stdlib_rm /tmp/ftzz` | 1.168 ± 0.009 |   1.155 |   1.183 | 1.01 ± 0.01 |
| `rm -r /tmp/ftzz`                      | 1.209 ± 0.012 |   1.192 |   1.223 | 1.05 ± 0.01 |
| `./target/release/rayon_rm /tmp/ftzz`  | 1.232 ± 0.013 |   1.218 |   1.266 | 1.07 ± 0.01 |

## Copy

### Setup

```bash
cargo install fcp xcp
git clone https://github.com/wheybags/wcp.git && mkdir wcp/build && (cd wcp/build && cmake .. -DCMAKE_BUILD_TYPE=Release && make)
```

### Run

```bash
for num_bytes in {0,100M}; do for num_files in {10,10_000,100_000,1M}; do hyperfine --warmup 3 -N --export-markdown "benches/copy_${num_files}_files_${num_bytes}_bytes.md" --export-json "benches/copy_${num_files}_files_${num_bytes}_bytes.json" --setup "ftzz g -n ${num_files} -b ${num_bytes} /tmp/ftzz" --prepare "rm -rf /tmp/ftzzz" --cleanup "rm -r /tmp/ftzz" "cp -r /tmp/ftzz /tmp/ftzzz" "fcp /tmp/ftzz /tmp/ftzzz" "xcp -r /tmp/ftzz /tmp/ftzzz" "./wcp/build/wcp /tmp/ftzz /tmp/ftzzz" "./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz" "./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz" "./target/release/cpz /tmp/ftzz /tmp/ftzzz"; done; done
for num_bytes in {0,100M}; do hyperfine --warmup 3 -N --export-markdown "benches/copy_100_000_files_${num_bytes}_bytes_0_depth.md" --export-json "benches/copy_100_000_files_${num_bytes}_bytes_0_depth.json" --setup "ftzz g -n 100_000 -b ${num_bytes} -d 0 /tmp/ftzz" --prepare "rm -rf /tmp/ftzzz" --cleanup "rm -r /tmp/ftzz" "cp -r /tmp/ftzz /tmp/ftzzz" "fcp /tmp/ftzz /tmp/ftzzz" "xcp -r /tmp/ftzz /tmp/ftzzz" "./wcp/build/wcp /tmp/ftzz /tmp/ftzzz" "./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz" "./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz" "./target/release/cpz /tmp/ftzz /tmp/ftzzz"; done
```

### Results

> Note: my benchmarking machine doesn't have io_uring enabled, so I was not able to include results
> with `wcp`. That said, I ran a few quick benches on my personal machine against `cpz` and found it
> to be faster than `wcp`.

#### `10_files_0_bytes.md`

| Command                                           | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------------------------------------------------|----------:|---------:|---------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 1.3 ± 0.1 |      1.1 |      1.7 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.5 ± 0.3 |      1.3 |      2.8 | 1.16 ± 0.23 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.9 ± 0.2 |      1.6 |      5.8 | 1.42 ± 0.17 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.9 ± 0.2 |      1.7 |      4.5 | 1.43 ± 0.18 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.1 ± 0.4 |      1.5 |      3.3 | 1.59 ± 0.31 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 2.2 ± 0.1 |      1.8 |      2.5 | 1.65 ± 0.14 |

#### `10_000_files_0_bytes.md`

| Command                                           |   Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------------------------------------------------|------------:|---------:|---------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  50.6 ± 1.3 |     48.2 |     52.4 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  66.3 ± 2.1 |     63.3 |     71.7 | 1.31 ± 0.05 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        |  67.8 ± 1.2 |     64.9 |     69.2 | 1.34 ± 0.04 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 112.2 ± 5.1 |    104.6 |    119.4 | 2.22 ± 0.12 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 233.1 ± 3.2 |    229.6 |    238.3 | 4.61 ± 0.14 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 243.5 ± 1.8 |    240.4 |    246.5 | 4.82 ± 0.13 |

#### `100_000_files_0_bytes.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 0.357 ± 0.002 |   0.354 |   0.361 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 0.486 ± 0.014 |   0.472 |   0.509 | 1.36 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 0.718 ± 0.013 |   0.702 |   0.747 | 2.01 ± 0.04 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 0.974 ± 0.010 |   0.961 |   0.989 | 2.73 ± 0.03 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.705 ± 0.022 |   1.678 |   1.755 | 4.78 ± 0.07 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.798 ± 0.015 |   1.777 |   1.824 | 5.04 ± 0.05 |

#### `1M_files_0_bytes.md`

| Command                                           |       Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------------------------------------|---------------:|--------:|--------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  5.438 ± 0.993 |   5.052 |   8.260 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  6.841 ± 0.361 |   6.579 |   7.569 | 1.26 ± 0.24 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 14.752 ± 0.064 |  14.664 |  14.838 | 2.71 ± 0.50 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 14.912 ± 0.088 |  14.836 |  15.099 | 2.74 ± 0.50 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 23.169 ± 0.067 |  23.088 |  23.314 | 4.26 ± 0.78 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 24.216 ± 0.085 |  24.103 |  24.364 | 4.45 ± 0.81 |

#### `10_files_100M_bytes.md`

| Command                                           | Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------------------------------------------------|----------:|---------:|---------:|------------:|
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.7 ± 0.2 |      1.5 |      3.9 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.9 ± 0.9 |      1.5 |     13.0 | 1.11 ± 0.55 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.0 ± 0.1 |      1.9 |      2.7 | 1.16 ± 0.13 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 2.1 ± 0.3 |      1.7 |      4.8 | 1.22 ± 0.22 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 2.3 ± 0.7 |      1.4 |     12.9 | 1.36 ± 0.41 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 3.1 ± 1.2 |      1.7 |      5.7 | 1.82 ± 0.72 |

#### `10_000_files_100M_bytes.md`

| Command                                           |   Mean [ms] | Min [ms] | Max [ms] |    Relative |
|:--------------------------------------------------|------------:|---------:|---------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  78.1 ± 5.2 |     75.0 |     92.3 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  97.0 ± 3.1 |     92.2 |    102.3 | 1.24 ± 0.09 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 102.0 ± 4.7 |     98.4 |    113.8 | 1.31 ± 0.11 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 151.0 ± 6.2 |    140.6 |    159.9 | 1.93 ± 0.15 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 344.1 ± 4.3 |    338.3 |    352.9 | 4.40 ± 0.30 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 349.9 ± 5.1 |    344.2 |    360.4 | 4.48 ± 0.31 |

#### `100_000_files_100M_bytes.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 0.628 ± 0.027 |   0.601 |   0.677 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 0.844 ± 0.052 |   0.763 |   0.946 | 1.34 ± 0.10 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 0.969 ± 0.028 |   0.948 |   1.025 | 1.54 ± 0.08 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.146 ± 0.015 |   1.124 |   1.174 | 1.82 ± 0.08 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.560 ± 0.026 |   2.518 |   2.597 | 4.07 ± 0.18 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.627 ± 0.022 |   2.581 |   2.652 | 4.18 ± 0.18 |

#### `1M_files_100M_bytes.md`

| Command                                           |       Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------------------------------------|---------------:|--------:|--------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  9.000 ± 0.206 |   8.651 |   9.245 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 11.894 ± 0.515 |  10.478 |  12.175 | 1.32 ± 0.06 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 16.869 ± 0.197 |  16.580 |  17.206 | 1.87 ± 0.05 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 17.139 ± 0.148 |  16.953 |  17.317 | 1.90 ± 0.05 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 35.392 ± 0.138 |  35.180 |  35.609 | 3.93 ± 0.09 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 35.703 ± 0.145 |  35.443 |  35.954 | 3.97 ± 0.09 |

#### `100_000_files_0_bytes_0_depth.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.296 ± 0.020 |   1.252 |   1.325 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.417 ± 0.010 |   1.401 |   1.433 | 1.09 ± 0.02 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.423 ± 0.010 |   1.409 |   1.440 | 1.10 ± 0.02 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 1.580 ± 0.011 |   1.566 |   1.604 | 1.22 ± 0.02 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.869 ± 0.018 |   1.850 |   1.902 | 1.44 ± 0.03 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.975 ± 0.010 |   1.957 |   1.991 | 1.52 ± 0.02 |

#### `100_000_files_100M_bytes_0_depth.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.405 ± 0.037 |   1.364 |   1.492 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.514 ± 0.010 |   1.502 |   1.527 | 1.08 ± 0.03 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.549 ± 0.033 |   1.515 |   1.617 | 1.10 ± 0.04 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 2.338 ± 0.012 |   2.317 |   2.360 | 1.66 ± 0.04 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.605 ± 0.023 |   2.582 |   2.658 | 1.85 ± 0.05 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.722 ± 0.012 |   2.703 |   2.735 | 1.94 ± 0.05 |
