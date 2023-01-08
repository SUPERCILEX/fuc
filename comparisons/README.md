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

#### `remove_10_files_0_bytes.md`

| Command                                | Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------|----------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 1.1 ± 0.1 |      1.0 |      2.5 |       0.6 |         1.3 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  | 1.5 ± 0.4 |      1.2 |      3.6 |       1.2 |         2.8 | 1.41 ± 0.43 |
| `./target/release/stdlib_rm /tmp/ftzz` | 2.5 ± 0.1 |      2.4 |      2.8 |       0.6 |         1.8 | 2.41 ± 0.17 |
| `rm -r /tmp/ftzz`                      | 3.5 ± 0.1 |      3.4 |      3.8 |       1.0 |         2.4 | 3.33 ± 0.22 |

#### `remove_10_000_files_0_bytes.md`

| Command                                |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  44.3 ± 1.6 |     42.4 |     47.7 |      12.3 |       205.9 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  53.9 ± 1.3 |     51.7 |     57.0 |      24.2 |       219.1 | 1.22 ± 0.05 |
| `./target/release/stdlib_rm /tmp/ftzz` | 106.2 ± 0.8 |    104.3 |    107.5 |       9.5 |        94.5 | 2.40 ± 0.09 |
| `rm -r /tmp/ftzz`                      | 117.8 ± 1.0 |    116.5 |    119.2 |      14.2 |       101.2 | 2.66 ± 0.10 |

#### `remove_100_000_files_0_bytes.md`

| Command                                |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  489.5 ± 61.7 |    382.0 |    574.4 |      64.8 |      1711.4 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  | 679.9 ± 109.8 |    540.9 |    855.5 |     170.9 |      2170.7 | 1.39 ± 0.28 |
| `./target/release/stdlib_rm /tmp/ftzz` |  835.8 ± 35.6 |    803.3 |    925.4 |      37.8 |       740.2 | 1.71 ± 0.23 |
| `rm -r /tmp/ftzz`                      |  858.5 ± 42.0 |    820.9 |    941.8 |      53.5 |       752.5 | 1.75 ± 0.24 |

#### `remove_1M_files_0_bytes.md`

| Command                                |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  5.039 ± 0.083 |   4.913 |   5.167 |    0.658 |     22.489 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  6.546 ± 0.517 |   5.957 |   7.621 |    1.452 |     23.319 | 1.30 ± 0.10 |
| `./target/release/stdlib_rm /tmp/ftzz` | 11.566 ± 0.045 |  11.502 |  11.663 |    0.467 |     10.816 | 2.30 ± 0.04 |
| `rm -r /tmp/ftzz`                      | 11.803 ± 0.055 |  11.718 |  11.919 |    0.584 |     10.932 | 2.34 ± 0.04 |

#### `remove_10_files_100M_bytes.md`

| Command                                |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------|-----------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rayon_rm /tmp/ftzz`  |  7.9 ± 0.2 |      7.4 |      8.7 |       1.4 |        47.3 |        1.00 |
| `./target/release/rmz /tmp/ftzz`       |  8.8 ± 0.3 |      8.2 |      9.5 |       0.7 |        29.8 | 1.11 ± 0.05 |
| `./target/release/stdlib_rm /tmp/ftzz` | 24.6 ± 0.1 |     24.3 |     24.9 |       0.2 |        24.1 | 3.10 ± 0.10 |
| `rm -r /tmp/ftzz`                      | 25.0 ± 0.4 |     24.5 |     26.5 |       0.4 |        24.3 | 3.16 ± 0.11 |

#### `remove_10_000_files_100M_bytes.md`

| Command                                |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  50.9 ± 1.4 |     49.2 |     53.4 |      13.6 |       286.4 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  63.1 ± 3.0 |     58.9 |     71.2 |      25.3 |       303.3 | 1.24 ± 0.07 |
| `./target/release/stdlib_rm /tmp/ftzz` | 153.5 ± 1.1 |    152.0 |    155.2 |       9.4 |       139.7 | 3.01 ± 0.08 |
| `rm -r /tmp/ftzz`                      | 165.6 ± 1.5 |    162.7 |    167.7 |      13.2 |       147.8 | 3.25 ± 0.09 |

#### `remove_100_000_files_100M_bytes.md`

| Command                                |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 0.387 ± 0.032 |   0.356 |   0.461 |    0.058 |      1.946 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  | 0.666 ± 0.152 |   0.476 |   0.855 |    0.163 |      2.543 | 1.72 ± 0.42 |
| `./target/release/stdlib_rm /tmp/ftzz` | 1.010 ± 0.004 |   1.003 |   1.016 |    0.046 |      0.937 | 2.61 ± 0.22 |
| `rm -r /tmp/ftzz`                      | 1.050 ± 0.036 |   1.024 |   1.147 |    0.053 |      0.955 | 2.72 ± 0.24 |

#### `remove_1M_files_100M_bytes.md`

| Command                                |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  7.598 ± 0.747 |   7.125 |   9.642 |    0.664 |     36.399 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`  |  9.249 ± 0.512 |   8.656 |  10.282 |    1.585 |     38.137 | 1.22 ± 0.14 |
| `./target/release/stdlib_rm /tmp/ftzz` | 27.119 ± 1.288 |  24.922 |  28.501 |    0.540 |     21.957 | 3.57 ± 0.39 |
| `rm -r /tmp/ftzz`                      | 27.399 ± 1.077 |  25.293 |  29.072 |    0.689 |     22.168 | 3.61 ± 0.38 |

#### `remove_100_000_files_0_bytes_0_depth.md`

| Command                                |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rmz /tmp/ftzz`       |  929.1 ± 10.5 |    913.3 |    941.5 |      30.2 |       881.7 |        1.00 |
| `./target/release/stdlib_rm /tmp/ftzz` |   935.3 ± 8.1 |    923.4 |    947.8 |      40.8 |       876.4 | 1.01 ± 0.01 |
| `rm -r /tmp/ftzz`                      |   967.1 ± 5.6 |    959.5 |    976.1 |      58.6 |       890.3 | 1.04 ± 0.01 |
| `./target/release/rayon_rm /tmp/ftzz`  | 1278.5 ± 43.7 |   1238.8 |   1396.8 |     105.3 |      2190.1 | 1.38 ± 0.05 |

#### `remove_100_000_files_100M_bytes_0_depth.md`

| Command                                |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/rmz /tmp/ftzz`       | 1.154 ± 0.017 |   1.133 |   1.183 |    0.036 |      1.093 |        1.00 |
| `./target/release/stdlib_rm /tmp/ftzz` | 1.162 ± 0.020 |   1.137 |   1.191 |    0.041 |      1.096 | 1.01 ± 0.02 |
| `rm -r /tmp/ftzz`                      | 1.181 ± 0.007 |   1.167 |   1.190 |    0.059 |      1.101 | 1.02 ± 0.02 |
| `./target/release/rayon_rm /tmp/ftzz`  | 1.242 ± 0.012 |   1.225 |   1.261 |    0.114 |      2.734 | 1.08 ± 0.02 |

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

#### `copy_10_files_0_bytes.md`

| Command                                           | Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:--------------------------------------------------|----------:|---------:|---------:|----------:|------------:|------------:|
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.7 ± 0.1 |      1.6 |      4.6 |       0.5 |         1.2 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 2.2 ± 0.4 |      1.9 |      4.1 |       1.8 |         4.6 | 1.26 ± 0.23 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 2.7 ± 0.7 |      1.8 |      3.9 |       2.1 |         5.8 | 1.55 ± 0.42 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 2.8 ± 0.2 |      2.6 |      3.4 |       1.2 |         4.0 | 1.65 ± 0.13 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 3.1 ± 0.1 |      3.0 |      3.7 |       0.7 |         2.2 | 1.80 ± 0.11 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 4.2 ± 0.3 |      3.6 |      5.1 |       2.8 |         5.0 | 2.41 ± 0.24 |

#### `copy_10_000_files_0_bytes.md`

| Command                                           |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:--------------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  49.1 ± 0.9 |     47.2 |     50.6 |      42.4 |       263.9 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  64.7 ± 1.6 |     62.2 |     69.3 |      81.6 |       348.7 | 1.32 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        |  66.2 ± 1.2 |     64.3 |     68.1 |      88.1 |       357.6 | 1.35 ± 0.03 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 108.1 ± 5.1 |    102.1 |    116.6 |      84.2 |       385.9 | 2.20 ± 0.11 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 230.0 ± 1.8 |    226.7 |    232.6 |      43.1 |       181.8 | 4.69 ± 0.10 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 241.0 ± 2.5 |    237.3 |    245.0 |      49.3 |       186.2 | 4.91 ± 0.11 |

#### `copy_100_000_files_0_bytes.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 0.360 ± 0.012 |   0.346 |   0.378 |    0.270 |      1.980 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 0.481 ± 0.043 |   0.459 |   0.600 |    0.551 |      2.618 | 1.34 ± 0.13 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 0.711 ± 0.007 |   0.700 |   0.724 |    0.625 |      4.195 | 1.97 ± 0.07 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 0.972 ± 0.019 |   0.943 |   1.013 |    0.541 |      3.373 | 2.70 ± 0.10 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.672 ± 0.012 |   1.653 |   1.691 |    0.287 |      1.351 | 4.64 ± 0.15 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.778 ± 0.007 |   1.769 |   1.793 |    0.317 |      1.427 | 4.94 ± 0.16 |

#### `copy_1M_files_0_bytes.md`

| Command                                           |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  6.011 ± 1.378 |   4.956 |   8.227 |    3.386 |     30.000 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  6.952 ± 0.753 |   6.459 |   8.592 |    6.498 |     36.905 | 1.16 ± 0.29 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 14.519 ± 0.066 |  14.442 |  14.659 |    6.508 |     47.911 | 2.42 ± 0.55 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 14.995 ± 0.099 |  14.875 |  15.227 |    6.507 |     75.961 | 2.49 ± 0.57 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 22.875 ± 0.082 |  22.728 |  22.985 |    3.453 |     19.023 | 3.81 ± 0.87 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 23.941 ± 0.125 |  23.753 |  24.172 |    3.857 |     19.678 | 3.98 ± 0.91 |

#### `copy_10_files_100M_bytes.md`

| Command                                           | Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:--------------------------------------------------|----------:|---------:|---------:|----------:|------------:|------------:|
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.5 ± 0.3 |      1.3 |      8.4 |       1.2 |         3.5 |        1.00 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.7 ± 0.2 |      1.5 |      3.8 |       0.4 |         1.2 | 1.08 ± 0.21 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 1.7 ± 0.7 |      1.2 |      4.7 |       1.0 |         2.2 | 1.08 ± 0.49 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.7 ± 0.7 |      1.3 |      8.1 |       1.3 |         3.8 | 1.09 ± 0.46 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.9 ± 0.2 |      1.8 |      8.4 |       0.4 |         1.4 | 1.25 ± 0.26 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 3.9 ± 0.5 |      3.1 |      5.1 |       2.6 |         5.5 | 2.57 ± 0.56 |

#### `copy_10_000_files_100M_bytes.md`

| Command                                           |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:--------------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  78.3 ± 3.9 |     74.3 |     87.3 |      50.5 |       454.5 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  93.9 ± 2.3 |     89.3 |     97.2 |      85.1 |       544.1 | 1.20 ± 0.07 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        |  97.5 ± 2.5 |     93.3 |    102.6 |      96.0 |       555.7 | 1.25 ± 0.07 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 139.1 ± 4.7 |    133.8 |    147.0 |      84.1 |       495.6 | 1.78 ± 0.11 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 328.8 ± 2.5 |    325.2 |    332.4 |      46.1 |       274.5 | 4.20 ± 0.21 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 345.9 ± 5.1 |    339.4 |    358.2 |      50.2 |       287.3 | 4.42 ± 0.23 |

#### `copy_100_000_files_100M_bytes.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 0.645 ± 0.056 |   0.603 |   0.791 |    0.322 |      3.502 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 0.743 ± 0.008 |   0.731 |   0.761 |    0.534 |      4.142 | 1.15 ± 0.10 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 0.956 ± 0.021 |   0.931 |   0.985 |    0.678 |      5.337 | 1.48 ± 0.13 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.124 ± 0.019 |   1.104 |   1.162 |    0.625 |      3.958 | 1.74 ± 0.16 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.515 ± 0.023 |   2.473 |   2.548 |    0.302 |      2.153 | 3.90 ± 0.34 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.598 ± 0.025 |   2.552 |   2.636 |    0.329 |      2.207 | 4.03 ± 0.35 |

#### `copy_1M_files_100M_bytes.md`

| Command                                           |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       |  8.945 ± 0.186 |   8.570 |   9.208 |    3.844 |     48.175 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 10.402 ± 0.326 |  10.101 |  11.001 |    6.620 |     55.300 | 1.16 ± 0.04 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 16.702 ± 0.206 |  16.385 |  16.999 |    7.600 |     57.872 | 1.87 ± 0.05 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 16.994 ± 0.178 |  16.757 |  17.250 |    7.127 |     85.346 | 1.90 ± 0.04 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 34.866 ± 0.097 |  34.736 |  35.078 |    3.652 |     30.420 | 3.90 ± 0.08 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 35.475 ± 0.132 |  35.278 |  35.654 |    4.096 |     30.585 | 3.97 ± 0.08 |

#### `copy_100_000_files_0_bytes_0_depth.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.279 ± 0.015 |   1.255 |   1.295 |    0.527 |      4.026 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.418 ± 0.009 |   1.404 |   1.431 |    0.451 |      7.203 | 1.11 ± 0.01 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.435 ± 0.019 |   1.416 |   1.484 |    0.441 |      7.483 | 1.12 ± 0.02 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 1.559 ± 0.016 |   1.538 |   1.600 |    0.188 |      1.338 | 1.22 ± 0.02 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.871 ± 0.031 |   1.837 |   1.931 |    0.292 |      1.545 | 1.46 ± 0.03 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.952 ± 0.010 |   1.938 |   1.968 |    0.311 |      1.607 | 1.53 ± 0.02 |

#### `copy_100_000_files_100M_bytes_0_depth.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.378 ± 0.052 |   1.310 |   1.489 |    0.591 |      4.796 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.521 ± 0.025 |   1.501 |   1.584 |    0.454 |      7.893 | 1.10 ± 0.05 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.522 ± 0.013 |   1.507 |   1.546 |    0.488 |      7.552 | 1.10 ± 0.04 |
| `./target/release/cpz /tmp/ftzz /tmp/ftzzz`       | 2.355 ± 0.010 |   2.338 |   2.369 |    0.223 |      2.072 | 1.71 ± 0.06 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.582 ± 0.015 |   2.562 |   2.609 |    0.297 |      2.227 | 1.87 ± 0.07 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.687 ± 0.009 |   2.673 |   2.704 |    0.326 |      2.300 | 1.95 ± 0.07 |
