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
mkdir /tmp/empty
for num_bytes in {0,100M}; do for num_files in {10,10_000,100_000,1M}; do hyperfine --warmup 3 -N --export-markdown "benches/remove_${num_files}_files_${num_bytes}_bytes.md" --export-json "benches/remove_${num_files}_files_${num_bytes}_bytes.json" --prepare "ftzz g -n ${num_files} -b ${num_bytes} /tmp/ftzz" "rm -r /tmp/ftzz" "find /tmp/ftzz -delete" "rsync --delete -a /tmp/empty/ /tmp/ftzz" "./target/release/stdlib_rm /tmp/ftzz" "./target/release/rayon_rm /tmp/ftzz" "./target/release/rmz /tmp/ftzz"; done; done
for num_bytes in {0,100M}; do hyperfine --warmup 3 -N --export-markdown "benches/remove_100_000_files_${num_bytes}_bytes_0_depth.md" --export-json "benches/remove_100_000_files_${num_bytes}_bytes_0_depth.json" --prepare "ftzz g -n 100_000 -b ${num_bytes} -d 0 /tmp/ftzz" "rm -r /tmp/ftzz" "find /tmp/ftzz -delete" "rsync --delete -a /tmp/empty/ /tmp/ftzz" "perl -e 'for(</tmp/ftzz/*>){unlink}'" "./target/release/stdlib_rm /tmp/ftzz" "./target/release/rayon_rm /tmp/ftzz" "./target/release/rmz /tmp/ftzz"; done
```

### Results

#### `remove_10_files_0_bytes.md`

| Command                                   |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*        |  1.1 ± 0.1 |      1.0 |      1.4 |       0.6 |         1.4 |         1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     |  1.4 ± 0.3 |      1.2 |      3.5 |       1.0 |         2.8 |  1.33 ± 0.26 |
| `./target/release/stdlib_rm /tmp/ftzz`    |  2.7 ± 0.0 |      2.6 |      2.9 |       0.6 |         1.9 |  2.49 ± 0.14 |
| `rm -r /tmp/ftzz`                         |  3.5 ± 0.1 |      3.4 |      3.7 |       1.0 |         2.4 |  3.26 ± 0.19 |
| `find /tmp/ftzz -delete`                  |  4.4 ± 0.1 |      4.2 |      4.6 |       1.1 |         2.7 |  4.07 ± 0.24 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 48.9 ± 0.1 |     48.7 |     49.1 |       3.6 |         6.0 | 45.49 ± 2.51 |

#### `remove_10_000_files_0_bytes.md`

| Command                                   |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        |  44.5 ± 1.8 |     42.4 |     49.7 |      10.5 |       207.3 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     |  54.0 ± 1.7 |     52.0 |     58.5 |      24.7 |       218.6 | 1.22 ± 0.06 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 106.4 ± 0.9 |    104.9 |    108.1 |      10.0 |        94.2 | 2.39 ± 0.10 |
| `find /tmp/ftzz -delete`                  | 116.8 ± 1.3 |    115.1 |    120.9 |      15.8 |        98.2 | 2.63 ± 0.11 |
| `rm -r /tmp/ftzz`                         | 117.8 ± 0.8 |    116.4 |    119.0 |      14.8 |       100.5 | 2.65 ± 0.11 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 179.8 ± 0.8 |    177.8 |    180.6 |      21.3 |       117.3 | 4.04 ± 0.16 |

#### `remove_100_000_files_0_bytes.md`

| Command                                   |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        |  481.6 ± 85.2 |    337.1 |    594.8 |      63.8 |      1743.7 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     | 753.6 ± 116.3 |    571.9 |    924.3 |     183.0 |      2364.7 | 1.56 ± 0.37 |
| `./target/release/stdlib_rm /tmp/ftzz`    |  839.3 ± 27.3 |    801.8 |    892.3 |      38.4 |       740.7 | 1.74 ± 0.31 |
| `find /tmp/ftzz -delete`                  |  849.9 ± 29.5 |    803.1 |    907.8 |      56.1 |       738.3 | 1.76 ± 0.32 |
| `rm -r /tmp/ftzz`                         |  873.4 ± 38.9 |    832.3 |    954.1 |      49.0 |       756.5 | 1.81 ± 0.33 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 1009.2 ± 12.4 |    996.1 |   1029.2 |      93.6 |       851.3 | 2.10 ± 0.37 |

#### `remove_1M_files_0_bytes.md`

| Command                                   |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        |  5.336 ± 0.640 |   4.929 |   6.870 |    0.655 |     23.003 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     |  6.448 ± 0.445 |   6.119 |   7.544 |    1.414 |     23.285 | 1.21 ± 0.17 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 11.648 ± 0.094 |  11.553 |  11.861 |    0.470 |     10.883 | 2.18 ± 0.26 |
| `find /tmp/ftzz -delete`                  | 11.781 ± 0.058 |  11.701 |  11.905 |    0.587 |     10.904 | 2.21 ± 0.26 |
| `rm -r /tmp/ftzz`                         | 11.851 ± 0.060 |  11.758 |  11.958 |    0.578 |     10.976 | 2.22 ± 0.27 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 13.717 ± 0.090 |  13.582 |  13.876 |    1.140 |     12.289 | 2.57 ± 0.31 |

#### `remove_10_files_100M_bytes.md`

| Command                                   |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------|-----------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rayon_rm /tmp/ftzz`     |  8.4 ± 0.2 |      8.0 |      9.0 |       1.4 |        51.0 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*        |  9.1 ± 0.2 |      8.5 |      9.6 |       0.6 |        31.2 | 1.08 ± 0.04 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 25.3 ± 0.2 |     25.0 |     25.9 |       0.2 |        24.8 | 3.02 ± 0.09 |
| `rm -r /tmp/ftzz`                         | 25.9 ± 0.4 |     25.3 |     26.6 |       0.4 |        25.1 | 3.08 ± 0.10 |
| `find /tmp/ftzz -delete`                  | 26.6 ± 1.9 |     25.9 |     40.5 |       0.4 |        25.2 | 3.17 ± 0.24 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 74.1 ± 1.7 |     71.8 |     79.5 |       2.1 |        32.0 | 8.82 ± 0.32 |

#### `remove_10_000_files_100M_bytes.md`

| Command                                   |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        |  52.0 ± 2.1 |     49.2 |     57.2 |      12.5 |       288.6 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     |  62.2 ± 1.3 |     59.1 |     64.1 |      25.0 |       306.1 | 1.20 ± 0.05 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 154.3 ± 1.5 |    151.6 |    157.1 |       8.6 |       141.3 | 2.97 ± 0.12 |
| `find /tmp/ftzz -delete`                  | 165.1 ± 0.8 |    164.0 |    166.6 |      14.7 |       145.6 | 3.18 ± 0.13 |
| `rm -r /tmp/ftzz`                         | 166.3 ± 1.3 |    163.6 |    168.1 |      15.1 |       146.6 | 3.20 ± 0.13 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 229.7 ± 1.1 |    228.1 |    231.2 |      19.1 |       167.3 | 4.42 ± 0.18 |

#### `remove_100_000_files_100M_bytes.md`

| Command                                   |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        | 0.376 ± 0.031 |   0.354 |   0.456 |    0.050 |      1.939 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     | 0.655 ± 0.156 |   0.441 |   0.831 |    0.157 |      2.488 | 1.74 ± 0.44 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 1.019 ± 0.013 |   1.004 |   1.050 |    0.044 |      0.946 | 2.71 ± 0.23 |
| `find /tmp/ftzz -delete`                  | 1.033 ± 0.010 |   1.021 |   1.046 |    0.054 |      0.947 | 2.75 ± 0.23 |
| `rm -r /tmp/ftzz`                         | 1.059 ± 0.034 |   1.027 |   1.121 |    0.050 |      0.972 | 2.82 ± 0.25 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 1.216 ± 0.006 |   1.208 |   1.226 |    0.098 |      1.061 | 3.24 ± 0.27 |

#### `remove_1M_files_100M_bytes.md`

| Command                                   |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        |  7.570 ± 0.158 |   7.307 |   7.776 |    0.642 |     36.387 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     |  9.506 ± 1.082 |   8.677 |  11.951 |    1.620 |     39.153 | 1.26 ± 0.15 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 23.826 ± 0.383 |  23.080 |  24.392 |    1.227 |     20.349 | 3.15 ± 0.08 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 27.119 ± 1.024 |  25.723 |  28.766 |    0.559 |     22.035 | 3.58 ± 0.15 |
| `find /tmp/ftzz -delete`                  | 27.457 ± 1.118 |  25.960 |  29.451 |    0.708 |     22.040 | 3.63 ± 0.17 |
| `rm -r /tmp/ftzz`                         | 27.878 ± 1.234 |  25.028 |  29.101 |    0.689 |     22.382 | 3.68 ± 0.18 |

#### `remove_100_000_files_0_bytes_0_depth.md`

| Command                                   |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        |   933.9 ± 7.9 |    921.4 |    947.3 |      29.1 |       886.4 |        1.00 |
| `./target/release/stdlib_rm /tmp/ftzz`    |   941.9 ± 9.6 |    928.3 |    957.6 |      38.4 |       883.6 | 1.01 ± 0.01 |
| `rm -r /tmp/ftzz`                         |   966.3 ± 6.7 |    960.2 |    983.9 |      58.4 |       889.3 | 1.03 ± 0.01 |
| `find /tmp/ftzz -delete`                  |   968.7 ± 9.1 |    956.8 |    987.3 |      60.7 |       888.1 | 1.04 ± 0.01 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 1089.1 ± 11.3 |   1078.9 |   1109.8 |     102.1 |       930.0 | 1.17 ± 0.02 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`    | 1210.5 ± 11.3 |   1200.7 |   1240.8 |     147.9 |      1042.5 | 1.30 ± 0.02 |
| `./target/release/rayon_rm /tmp/ftzz`     | 1270.8 ± 10.8 |   1260.0 |   1287.1 |     103.9 |      2176.9 | 1.36 ± 0.02 |

#### `remove_100_000_files_100M_bytes_0_depth.md`

| Command                                   |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        | 1.155 ± 0.014 |   1.133 |   1.175 |    0.037 |      1.091 |        1.00 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 1.170 ± 0.011 |   1.152 |   1.185 |    0.042 |      1.098 | 1.01 ± 0.02 |
| `find /tmp/ftzz -delete`                  | 1.180 ± 0.015 |   1.146 |   1.199 |    0.065 |      1.093 | 1.02 ± 0.02 |
| `rm -r /tmp/ftzz`                         | 1.186 ± 0.014 |   1.165 |   1.208 |    0.067 |      1.098 | 1.03 ± 0.02 |
| `./target/release/rayon_rm /tmp/ftzz`     | 1.256 ± 0.013 |   1.237 |   1.281 |    0.113 |      2.749 | 1.09 ± 0.02 |
| `rsync --delete -a /tmp/empty/ /tmp/ftzz` | 1.303 ± 0.004 |   1.297 |   1.311 |    0.103 |      1.147 | 1.13 ± 0.01 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`    | 1.439 ± 0.008 |   1.428 |   1.458 |    0.151 |      1.268 | 1.25 ± 0.02 |

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
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     | 2.8 ± 0.2 |      2.6 |      3.4 |       1.2 |         4.0 | 1.65 ± 0.13 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 3.1 ± 0.1 |      3.0 |      3.7 |       0.7 |         2.2 | 1.80 ± 0.11 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 4.2 ± 0.3 |      3.6 |      5.1 |       2.8 |         5.0 | 2.41 ± 0.24 |

#### `copy_10_000_files_0_bytes.md`

| Command                                           |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:--------------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     |  49.1 ± 0.9 |     47.2 |     50.6 |      42.4 |       263.9 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  64.7 ± 1.6 |     62.2 |     69.3 |      81.6 |       348.7 | 1.32 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        |  66.2 ± 1.2 |     64.3 |     68.1 |      88.1 |       357.6 | 1.35 ± 0.03 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 108.1 ± 5.1 |    102.1 |    116.6 |      84.2 |       385.9 | 2.20 ± 0.11 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 230.0 ± 1.8 |    226.7 |    232.6 |      43.1 |       181.8 | 4.69 ± 0.10 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 241.0 ± 2.5 |    237.3 |    245.0 |      49.3 |       186.2 | 4.91 ± 0.11 |

#### `copy_100_000_files_0_bytes.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     | 0.360 ± 0.012 |   0.346 |   0.378 |    0.270 |      1.980 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 0.481 ± 0.043 |   0.459 |   0.600 |    0.551 |      2.618 | 1.34 ± 0.13 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 0.711 ± 0.007 |   0.700 |   0.724 |    0.625 |      4.195 | 1.97 ± 0.07 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 0.972 ± 0.019 |   0.943 |   1.013 |    0.541 |      3.373 | 2.70 ± 0.10 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.672 ± 0.012 |   1.653 |   1.691 |    0.287 |      1.351 | 4.64 ± 0.15 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.778 ± 0.007 |   1.769 |   1.793 |    0.317 |      1.427 | 4.94 ± 0.16 |

#### `copy_1M_files_0_bytes.md`

| Command                                           |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     |  6.011 ± 1.378 |   4.956 |   8.227 |    3.386 |     30.000 |        1.00 |
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
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     | 1.7 ± 0.7 |      1.2 |      4.7 |       1.0 |         2.2 | 1.08 ± 0.49 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.7 ± 0.7 |      1.3 |      8.1 |       1.3 |         3.8 | 1.09 ± 0.46 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.9 ± 0.2 |      1.8 |      8.4 |       0.4 |         1.4 | 1.25 ± 0.26 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 3.9 ± 0.5 |      3.1 |      5.1 |       2.6 |         5.5 | 2.57 ± 0.56 |

#### `copy_10_000_files_100M_bytes.md`

| Command                                           |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:--------------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     |  78.3 ± 3.9 |     74.3 |     87.3 |      50.5 |       454.5 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  |  93.9 ± 2.3 |     89.3 |     97.2 |      85.1 |       544.1 | 1.20 ± 0.07 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        |  97.5 ± 2.5 |     93.3 |    102.6 |      96.0 |       555.7 | 1.25 ± 0.07 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 139.1 ± 4.7 |    133.8 |    147.0 |      84.1 |       495.6 | 1.78 ± 0.11 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 328.8 ± 2.5 |    325.2 |    332.4 |      46.1 |       274.5 | 4.20 ± 0.21 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 345.9 ± 5.1 |    339.4 |    358.2 |      50.2 |       287.3 | 4.42 ± 0.23 |

#### `copy_100_000_files_100M_bytes.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     | 0.645 ± 0.056 |   0.603 |   0.791 |    0.322 |      3.502 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 0.743 ± 0.008 |   0.731 |   0.761 |    0.534 |      4.142 | 1.15 ± 0.10 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 0.956 ± 0.021 |   0.931 |   0.985 |    0.678 |      5.337 | 1.48 ± 0.13 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.124 ± 0.019 |   1.104 |   1.162 |    0.625 |      3.958 | 1.74 ± 0.16 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.515 ± 0.023 |   2.473 |   2.548 |    0.302 |      2.153 | 3.90 ± 0.34 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.598 ± 0.025 |   2.552 |   2.636 |    0.329 |      2.207 | 4.03 ± 0.35 |

#### `copy_1M_files_100M_bytes.md`

| Command                                           |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     |  8.945 ± 0.186 |   8.570 |   9.208 |    3.844 |     48.175 |        1.00 |
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
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     | 1.559 ± 0.016 |   1.538 |   1.600 |    0.188 |      1.338 | 1.22 ± 0.02 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 1.871 ± 0.031 |   1.837 |   1.931 |    0.292 |      1.545 | 1.46 ± 0.03 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 1.952 ± 0.010 |   1.938 |   1.968 |    0.311 |      1.607 | 1.53 ± 0.02 |

#### `copy_100_000_files_100M_bytes_0_depth.md`

| Command                                           |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:--------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                     | 1.378 ± 0.052 |   1.310 |   1.489 |    0.591 |      4.796 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`  | 1.521 ± 0.025 |   1.501 |   1.584 |    0.454 |      7.893 | 1.10 ± 0.05 |
| `fcp /tmp/ftzz /tmp/ftzzz`                        | 1.522 ± 0.013 |   1.507 |   1.546 |    0.488 |      7.552 | 1.10 ± 0.04 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*     | 2.355 ± 0.010 |   2.338 |   2.369 |    0.223 |      2.072 | 1.71 ± 0.06 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                      | 2.582 ± 0.015 |   2.562 |   2.609 |    0.297 |      2.227 | 1.87 ± 0.07 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz` | 2.687 ± 0.009 |   2.673 |   2.704 |    0.326 |      2.300 | 1.95 ± 0.07 |
