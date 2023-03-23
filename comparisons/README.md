# Benchmarks

The benchmarks take several hours to run and create hundreds of millions of files. Run at your own
risk. :)

## Setup

Run all commands in the root of this repository.

```bash
cargo install hyperfine ftzz
cargo b --workspace --release
mkdir benches /tmp/empty
```

### My environment

- Linux: 6.2.6-76060206-generic
- CPU: Intel i7-10875H
- Memory: 64038MiB
- `/tmp` was mounted as a `tmpfs /tmp tmpfs rw,nosuid,nodev,size=32787840k,inode64 0 0` to not
  destroy my laptop's SSD. Thus, the results implicitly assume zero-overhead I/O. I've found the
  relative scaling position to be mostly consistent when benchmarked on a real FS while the relative
  numbers are exaggerated (e.g. something 10x slower is probably 4x slower in reality).

## Remove

### Run

```bash
for num_bytes in {0,1G}; do
  for num_files in {10,10_000,100_000,1M}; do
    hyperfine --warmup 3 -N \
      --export-markdown "benches/remove_${num_files}_files_${num_bytes}_bytes.md" \
      --export-json "benches/remove_${num_files}_files_${num_bytes}_bytes.json" \
      --prepare "ftzz g -n ${num_files} -b ${num_bytes} /tmp/ftzz" \
        "rm -r /tmp/ftzz" \
        "find /tmp/ftzz -delete" \
        "rsync --delete -r /tmp/empty/ /tmp/ftzz" \
        "./target/release/rm_stdlib /tmp/ftzz" \
        "./target/release/rm_rayon /tmp/ftzz" \
        "./target/release/rm_remove_dir_all /tmp/ftzz" \
        "./target/release/rmz /tmp/ftzz"
  done

  hyperfine --warmup 3 -N \
    --export-markdown "benches/remove_100_000_files_${num_bytes}_bytes_0_depth.md" \
    --export-json "benches/remove_100_000_files_${num_bytes}_bytes_0_depth.json" \
    --prepare "ftzz g -n 100_000 -b ${num_bytes} -d 0 /tmp/ftzz" \
      "rm -r /tmp/ftzz" \
      "find /tmp/ftzz -delete" \
      "rsync --delete -r /tmp/empty/ /tmp/ftzz" \
      "perl -e 'for(</tmp/ftzz/*>){unlink}'" \
      "./target/release/rm_stdlib /tmp/ftzz" \
      "./target/release/rm_rayon /tmp/ftzz" \
      "./target/release/rm_remove_dir_all /tmp/ftzz" \
      "./target/release/rmz /tmp/ftzz"

  hyperfine --warmup 3 -N \
    --export-markdown "benches/remove_100_000_files_${num_bytes}_bytes_5_files_per_dir.md" \
    --export-json "benches/remove_100_000_files_${num_bytes}_bytes_5_files_per_dir.json" \
    --prepare "ftzz g -n 100_000 -b ${num_bytes} -r 5 /tmp/ftzz" \
      "rm -r /tmp/ftzz" \
      "find /tmp/ftzz -delete" \
      "rsync --delete -r /tmp/empty/ /tmp/ftzz" \
      "./target/release/rm_stdlib /tmp/ftzz" \
      "./target/release/rm_rayon /tmp/ftzz" \
      "./target/release/rm_remove_dir_all /tmp/ftzz" \
      "./target/release/rmz /tmp/ftzz"
done
```

### Results

#### `remove_1M_files_0_bytes.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.233 ± 0.007 |   0.220 |   0.247 |    0.093 |      3.060 |         1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.267 ± 0.006 |   0.259 |   0.276 |    0.437 |      3.502 |  1.15 ± 0.04 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.658 ± 0.013 |   1.639 |   1.680 |    0.104 |      1.480 |  7.11 ± 0.21 |
| `find /tmp/ftzz -delete`                       | 1.723 ± 0.013 |   1.706 |   1.742 |    0.176 |      1.476 |  7.39 ± 0.22 |
| `rm -r /tmp/ftzz`                              | 1.780 ± 0.018 |   1.740 |   1.798 |    0.159 |      1.553 |  7.63 ± 0.23 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 2.812 ± 0.021 |   2.769 |   2.835 |    0.382 |      2.356 | 12.06 ± 0.36 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 3.324 ± 0.035 |   3.279 |   3.402 |    0.284 |      2.978 | 14.25 ± 0.44 |

#### `remove_1M_files_1G_bytes.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.321 ± 0.007 |   0.311 |   0.332 |    0.105 |      4.223 |         1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.457 ± 0.206 |   0.373 |   1.040 |    0.508 |      5.481 |  1.42 ± 0.64 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 2.228 ± 0.050 |   2.188 |   2.320 |    0.144 |      2.027 |  6.94 ± 0.22 |
| `find /tmp/ftzz -delete`                       | 2.345 ± 0.053 |   2.301 |   2.457 |    0.208 |      2.086 |  7.30 ± 0.23 |
| `rm -r /tmp/ftzz`                              | 2.398 ± 0.087 |   2.329 |   2.583 |    0.195 |      2.153 |  7.47 ± 0.32 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 3.486 ± 0.076 |   3.347 |   3.579 |    0.461 |      2.961 | 10.85 ± 0.34 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 3.802 ± 0.052 |   3.691 |   3.881 |    0.347 |      3.408 | 11.84 ± 0.31 |

#### `remove_100_000_files_0_bytes.md`

| Command                                        |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|-------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             |   24.6 ± 1.9 |     20.8 |     30.6 |      10.7 |       283.7 |         1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  47.9 ± 12.5 |     32.7 |     80.7 |      38.6 |       313.4 |  1.95 ± 0.53 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  165.8 ± 3.6 |    159.0 |    173.2 |      11.4 |       148.5 |  6.74 ± 0.55 |
| `find /tmp/ftzz -delete`                       |  174.0 ± 4.6 |    166.9 |    180.9 |      19.8 |       147.4 |  7.07 ± 0.58 |
| `rm -r /tmp/ftzz`                              |  182.8 ± 3.1 |    177.0 |    188.1 |      19.7 |       156.5 |  7.43 ± 0.60 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      |  320.1 ± 2.5 |    316.8 |    323.8 |      44.1 |       234.3 | 13.01 ± 1.02 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 336.3 ± 10.7 |    328.5 |    364.0 |      26.4 |       303.2 | 13.66 ± 1.15 |

#### `remove_100_000_files_0_bytes_0_depth.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rm_rayon /tmp/ftzz`          | 124.7 ± 1.5 |    121.6 |    127.1 |      30.0 |      1298.6 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             | 136.7 ± 3.1 |    131.1 |    143.8 |       4.1 |       126.0 | 1.10 ± 0.03 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 143.2 ± 5.3 |    133.1 |    147.8 |       8.8 |       128.4 | 1.15 ± 0.05 |
| `rm -r /tmp/ftzz`                              | 157.4 ± 3.5 |    151.5 |    164.3 |      12.7 |       139.5 | 1.26 ± 0.03 |
| `find /tmp/ftzz -delete`                       | 158.4 ± 3.3 |    152.4 |    162.8 |      13.5 |       139.1 | 1.27 ± 0.03 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 282.2 ± 3.5 |    278.4 |    289.6 |      42.2 |       195.8 | 2.26 ± 0.04 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 293.2 ± 4.2 |    287.4 |    299.5 |      25.6 |       261.7 | 2.35 ± 0.04 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`         | 328.9 ± 4.0 |    321.4 |    335.0 |      63.2 |       261.1 | 2.64 ± 0.05 |

#### `remove_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                        |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|-------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             |   34.4 ± 1.6 |     31.7 |     40.5 |      54.8 |       423.6 |         1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |   58.8 ± 5.5 |     48.4 |     69.4 |      89.2 |       417.5 |  1.71 ± 0.18 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 252.7 ± 11.7 |    245.2 |    283.4 |      15.2 |       230.7 |  7.35 ± 0.49 |
| `find /tmp/ftzz -delete`                       | 335.2 ± 35.8 |    305.0 |    413.8 |      44.9 |       284.4 |  9.75 ± 1.14 |
| `rm -r /tmp/ftzz`                              |  342.4 ± 9.7 |    329.9 |    364.2 |      37.3 |       298.7 |  9.96 ± 0.55 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  427.0 ± 9.3 |    416.2 |    441.5 |      41.4 |       379.0 | 12.42 ± 0.65 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 457.2 ± 24.8 |    438.9 |    522.3 |      64.1 |       349.9 | 13.30 ± 0.96 |

#### `remove_100_000_files_1G_bytes.md`

| Command                                        |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|-------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             |   47.9 ± 5.1 |     43.0 |     62.4 |      12.7 |       578.2 |         1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  69.8 ± 11.1 |     51.9 |     92.5 |      48.0 |       572.5 |  1.46 ± 0.28 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  334.1 ± 6.9 |    323.4 |    349.1 |      16.1 |       308.9 |  6.97 ± 0.76 |
| `find /tmp/ftzz -delete`                       |  355.2 ± 7.9 |    347.1 |    375.3 |      30.2 |       316.5 |  7.41 ± 0.81 |
| `rm -r /tmp/ftzz`                              | 396.9 ± 54.8 |    356.1 |    532.2 |      26.0 |       363.6 |  8.28 ± 1.44 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      |  503.4 ± 4.7 |    497.3 |    508.8 |      46.7 |       412.4 | 10.50 ± 1.12 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 523.0 ± 11.8 |    505.4 |    543.7 |      37.9 |       478.3 | 10.91 ± 1.19 |

#### `remove_100_000_files_1G_bytes_0_depth.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rm_rayon /tmp/ftzz`          | 140.1 ± 3.0 |    135.8 |    144.8 |      42.7 |      1596.3 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             | 296.1 ± 2.4 |    293.6 |    300.6 |      14.1 |       273.9 | 2.11 ± 0.05 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 301.7 ± 4.4 |    294.8 |    309.7 |      11.4 |       282.5 | 2.15 ± 0.06 |
| `rm -r /tmp/ftzz`                              | 316.8 ± 2.2 |    311.9 |    319.8 |      22.4 |       286.2 | 2.26 ± 0.05 |
| `find /tmp/ftzz -delete`                       | 321.5 ± 2.9 |    316.7 |    325.4 |      21.7 |       292.3 | 2.30 ± 0.05 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 434.8 ± 3.0 |    431.1 |    438.9 |      49.8 |       338.9 | 3.10 ± 0.07 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 452.3 ± 3.2 |    447.9 |    458.8 |      28.8 |       416.4 | 3.23 ± 0.07 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`         | 486.7 ± 5.8 |    477.1 |    493.7 |      65.0 |       415.2 | 3.47 ± 0.08 |

#### `remove_100_000_files_1G_bytes_5_files_per_dir.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  49.4 ± 1.8 |     47.5 |     53.2 |      35.0 |       682.1 |         1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  80.1 ± 5.2 |     67.5 |     91.7 |      95.7 |       677.2 |  1.62 ± 0.12 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 411.7 ± 3.5 |    406.7 |    418.7 |      23.9 |       377.3 |  8.33 ± 0.30 |
| `find /tmp/ftzz -delete`                       | 469.9 ± 3.4 |    464.7 |    476.3 |      48.7 |       412.3 |  9.51 ± 0.35 |
| `rm -r /tmp/ftzz`                              | 502.1 ± 5.9 |    496.4 |    513.8 |      48.2 |       445.6 | 10.16 ± 0.38 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 584.3 ± 5.0 |    576.0 |    594.6 |      45.9 |       530.7 | 11.83 ± 0.43 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 609.4 ± 3.2 |    604.1 |    613.6 |      75.7 |       490.6 | 12.33 ± 0.44 |

#### `remove_10_000_files_0_bytes.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             |   4.3 ± 0.8 |      3.4 |      7.4 |       6.4 |        32.0 |         1.00 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  19.1 ± 1.3 |     16.9 |     27.2 |       2.0 |        16.9 |  4.44 ± 0.83 |
| `find /tmp/ftzz -delete`                       |  22.8 ± 1.5 |     20.9 |     27.6 |       2.5 |        19.8 |  5.28 ± 0.98 |
| `rm -r /tmp/ftzz`                              |  24.2 ± 1.4 |     22.5 |     28.7 |       2.5 |        21.4 |  5.62 ± 1.03 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  36.7 ± 2.2 |     32.6 |     43.2 |       3.6 |        32.4 |  8.50 ± 1.56 |
| `./target/release/rm_rayon /tmp/ftzz`          | 39.4 ± 15.3 |      5.0 |     85.5 |       9.3 |        51.2 |  9.14 ± 3.89 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      |  85.4 ± 8.5 |     76.2 |    106.7 |      11.2 |        36.7 | 19.79 ± 3.97 |

#### `remove_10_000_files_1G_bytes.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  20.7 ± 0.6 |     19.3 |     22.7 |       3.3 |       264.5 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 44.4 ± 15.5 |     20.4 |     79.6 |       7.6 |       242.9 | 2.15 ± 0.75 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 109.3 ± 4.5 |    106.4 |    126.2 |       1.3 |       107.2 | 5.29 ± 0.27 |
| `find /tmp/ftzz -delete`                       | 111.3 ± 1.7 |    107.7 |    114.6 |       4.6 |       106.1 | 5.38 ± 0.18 |
| `rm -r /tmp/ftzz`                              | 113.9 ± 3.7 |    111.6 |    128.1 |       4.2 |       109.2 | 5.51 ± 0.24 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 125.4 ± 3.6 |    122.1 |    134.8 |       5.0 |       119.7 | 6.06 ± 0.25 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 165.1 ± 1.4 |    162.2 |    167.8 |      11.0 |       115.2 | 7.98 ± 0.24 |

#### `remove_10_files_0_bytes.md`

| Command                                        |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| `./target/release/rm_stdlib /tmp/ftzz`         |  0.7 ± 0.1 |      0.6 |      2.3 |       0.6 |         0.1 |         1.00 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  0.8 ± 0.1 |      0.7 |      2.5 |       0.6 |         0.1 |  1.09 ± 0.15 |
| *`./target/release/rmz /tmp/ftzz`*             |  0.9 ± 0.1 |      0.8 |      2.0 |       1.2 |         0.3 |  1.34 ± 0.17 |
| `rm -r /tmp/ftzz`                              |  1.0 ± 0.1 |      0.8 |      1.3 |       0.7 |         0.2 |  1.38 ± 0.17 |
| `find /tmp/ftzz -delete`                       |  1.2 ± 0.1 |      1.0 |      1.7 |       0.8 |         0.2 |  1.66 ± 0.20 |
| `./target/release/rm_rayon /tmp/ftzz`          |  1.4 ± 0.2 |      1.2 |      3.4 |       3.6 |         1.3 |  2.06 ± 0.30 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 46.4 ± 2.9 |     43.9 |     54.2 |       5.3 |         3.5 | 66.76 ± 7.71 |

#### `remove_10_files_1G_bytes.md`

| Command                                        |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|-----------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rm_rayon /tmp/ftzz`          | 16.8 ± 1.9 |     13.4 |     21.1 |       1.3 |        68.6 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             | 26.3 ± 1.4 |     24.7 |     31.6 |       0.2 |        54.2 | 1.56 ± 0.20 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 52.3 ± 3.7 |     47.7 |     67.2 |       0.0 |        52.1 | 3.11 ± 0.42 |
| `rm -r /tmp/ftzz`                              | 52.5 ± 2.5 |     49.7 |     63.8 |       0.1 |        52.1 | 3.12 ± 0.38 |
| `find /tmp/ftzz -delete`                       | 52.5 ± 1.8 |     50.3 |     60.2 |       0.2 |        52.1 | 3.12 ± 0.37 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 54.4 ± 5.1 |     50.4 |     75.5 |       0.1 |        53.9 | 3.24 ± 0.47 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 95.8 ± 1.2 |     93.5 |     97.6 |       2.3 |        55.1 | 5.70 ± 0.65 |

## Copy

### Setup

```bash
cargo install fcp xcp
git clone https://github.com/wheybags/wcp.git && mkdir wcp/build && (cd wcp/build && cmake .. -DCMAKE_BUILD_TYPE=Release && make)
```

### Run

```bash
for num_bytes in {0,1G}; do
  for num_files in {10,10_000,100_000,1M}; do
    hyperfine --warmup 3 -N \
      --export-markdown "benches/copy_${num_files}_files_${num_bytes}_bytes.md" \
      --export-json "benches/copy_${num_files}_files_${num_bytes}_bytes.json" \
      --setup "ftzz g -n ${num_files} -b ${num_bytes} /tmp/ftzz" \
      --prepare "rm -rf /tmp/ftzzz" --cleanup "rm -r /tmp/ftzz" \
        "cp -r /tmp/ftzz /tmp/ftzzz" \
        "fcp /tmp/ftzz /tmp/ftzzz" \
        "xcp -r /tmp/ftzz /tmp/ftzzz" \
        "./wcp/build/wcp /tmp/ftzz /tmp/ftzzz" \
        "rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz" \
        "sh -c '(cd /tmp/ftzz; tar cf - .) | (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'" \
        "./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz" \
        "./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz" \
        "./target/release/cpz /tmp/ftzz /tmp/ftzzz"
  done

  hyperfine --warmup 3 -N \
    --export-markdown "benches/copy_100_000_files_${num_bytes}_bytes_0_depth.md" \
    --export-json "benches/copy_100_000_files_${num_bytes}_bytes_0_depth.json" \
    --setup "ftzz g -n 100_000 -b ${num_bytes} -d 0 /tmp/ftzz" \
    --prepare "rm -rf /tmp/ftzzz" --cleanup "rm -r /tmp/ftzz" \
      "cp -r /tmp/ftzz /tmp/ftzzz" \
      "fcp /tmp/ftzz /tmp/ftzzz" \
      "xcp -r /tmp/ftzz /tmp/ftzzz" \
      "./wcp/build/wcp /tmp/ftzz /tmp/ftzzz" \
      "rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz" \
      "sh -c '(cd /tmp/ftzz; tar cf - .) | (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'" \
      "./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz" \
      "./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz" \
      "./target/release/cpz /tmp/ftzz /tmp/ftzzz"

  hyperfine --warmup 3 -N \
    --export-markdown "benches/copy_100_000_files_${num_bytes}_bytes_5_files_per_dir.md" \
    --export-json "benches/copy_100_000_files_${num_bytes}_bytes_5_files_per_dir.json" \
    --setup "ftzz g -n 100_000 -b ${num_bytes} -r 5 /tmp/ftzz" \
    --prepare "rm -rf /tmp/ftzzz" --cleanup "rm -r /tmp/ftzz" \
      "cp -r /tmp/ftzz /tmp/ftzzz" \
      "fcp /tmp/ftzz /tmp/ftzzz" \
      "xcp -r /tmp/ftzz /tmp/ftzzz" \
      "./wcp/build/wcp /tmp/ftzz /tmp/ftzzz" \
      "rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz" \
      "sh -c '(cd /tmp/ftzz; tar cf - .) | (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'" \
      "./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz" \
      "./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz" \
      "./target/release/cpz /tmp/ftzz /tmp/ftzzz"
done

hyperfine --warmup 3 -N \
  --export-markdown "benches/copy_100_000_files_1G_bytes_0_depth_0_entropy.md" \
  --export-json "benches/copy_100_000_files_1G_bytes_0_depth_0_entropy.json" \
  --setup "ftzz g -n 100_000 -b 1G -d 0 --fill-byte 0 /tmp/ftzz" \
  --prepare "rm -rf /tmp/ftzzz" --cleanup "rm -r /tmp/ftzz" \
    "cp -r /tmp/ftzz /tmp/ftzzz" \
    "fcp /tmp/ftzz /tmp/ftzzz" \
    "xcp -r /tmp/ftzz /tmp/ftzzz" \
    "./wcp/build/wcp /tmp/ftzz /tmp/ftzzz" \
    "rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz" \
    "sh -c '(cd /tmp/ftzz; tar cf - .) | (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'" \
    "./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz" \
    "./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz" \
    "./target/release/cpz /tmp/ftzz /tmp/ftzzz"
```

### Results

#### `copy_1M_files_0_bytes.md`

| Command                                                                             |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  0.565 ± 0.010 |   0.551 |   0.579 |    0.381 |      7.453 |         1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  0.911 ± 0.017 |   0.885 |   0.935 |    1.369 |     12.066 |  1.61 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  1.044 ± 0.020 |   1.007 |   1.084 |    1.774 |     13.847 |  1.85 ± 0.05 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  2.058 ± 0.073 |   1.978 |   2.228 |    1.830 |      8.296 |  3.64 ± 0.14 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  3.682 ± 0.072 |   3.552 |   3.792 |    1.750 |      3.893 |  6.52 ± 0.17 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              |  4.641 ± 0.047 |   4.537 |   4.693 |    2.859 |      3.503 |  8.22 ± 0.17 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  5.937 ± 0.030 |   5.889 |   5.978 |    0.691 |      5.242 | 10.51 ± 0.19 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  6.439 ± 0.071 |   6.366 |   6.589 |    0.810 |      5.623 | 11.40 ± 0.24 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 13.092 ± 0.147 |  12.866 |  13.291 |    2.657 |     11.959 | 23.19 ± 0.48 |

#### `copy_1M_files_1G_bytes.md`

| Command                                                                             |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  0.860 ± 0.016 |   0.839 |   0.882 |    0.436 |     11.505 |         1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  1.177 ± 0.015 |   1.156 |   1.201 |    1.571 |     15.999 |  1.37 ± 0.03 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  1.312 ± 0.029 |   1.255 |   1.355 |    1.983 |     17.511 |  1.53 ± 0.04 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  2.691 ± 0.027 |   2.641 |   2.730 |    1.939 |     10.653 |  3.13 ± 0.07 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  5.678 ± 0.062 |   5.619 |   5.817 |    2.321 |      7.612 |  6.60 ± 0.14 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  8.284 ± 0.064 |   8.232 |   8.430 |    0.821 |      7.452 |  9.63 ± 0.20 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  8.414 ± 0.079 |   8.305 |   8.540 |    1.037 |      7.362 |  9.78 ± 0.21 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              |  9.998 ± 0.155 |   9.788 |  10.284 |    2.738 |     16.684 | 11.62 ± 0.28 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 13.427 ± 0.213 |  13.219 |  13.839 |    3.752 |     15.649 | 15.61 ± 0.39 |

#### `copy_100_000_files_0_bytes.md`

| Command                                                                             |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------------------------------------------------|--------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |    57.2 ± 1.0 |     55.9 |     59.2 |      33.3 |       712.6 |         1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |   100.9 ± 4.0 |     93.2 |    104.1 |     187.0 |      1274.0 |  1.76 ± 0.08 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  117.0 ± 10.1 |    103.4 |    132.8 |     133.0 |      1172.3 |  2.05 ± 0.18 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |   196.3 ± 4.2 |    192.3 |    204.1 |     196.1 |       762.0 |  3.43 ± 0.09 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |   361.4 ± 2.6 |    357.8 |    365.7 |     172.8 |       384.9 |  6.32 ± 0.12 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              |   532.7 ± 8.0 |    515.5 |    541.6 |     360.1 |       364.1 |  9.31 ± 0.21 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |   587.9 ± 3.5 |    582.5 |    591.7 |      79.3 |       506.8 | 10.28 ± 0.19 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |   639.7 ± 3.0 |    634.6 |    644.3 |      85.9 |       553.1 | 11.18 ± 0.20 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 1479.2 ± 11.2 |   1467.2 |   1498.2 |     239.1 |      1204.3 | 25.86 ± 0.48 |

#### `copy_100_000_files_0_bytes_0_depth.md`

| Command                                                                             |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------------------------------------------------|-------------:|---------:|---------:|----------:|------------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  172.3 ± 5.4 |    166.7 |    183.6 |     155.7 |       672.2 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  193.3 ± 2.7 |    190.2 |    198.1 |      99.3 |      2405.8 | 1.12 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  208.6 ± 2.0 |    206.7 |    213.0 |     105.3 |      2343.8 | 1.21 ± 0.04 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  324.9 ± 3.2 |    320.3 |    329.2 |     163.8 |       323.0 | 1.89 ± 0.06 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  383.3 ± 2.1 |    380.4 |    386.7 |      24.0 |       358.9 | 2.22 ± 0.07 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 440.0 ± 10.2 |    421.9 |    459.3 |     291.2 |       296.4 | 2.55 ± 0.10 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  496.3 ± 5.3 |    486.7 |    503.7 |      58.3 |       437.1 | 2.88 ± 0.09 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  533.5 ± 2.7 |    530.1 |    538.2 |      70.3 |       462.5 | 3.10 ± 0.10 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         |  945.0 ± 6.7 |    935.2 |    954.8 |     203.4 |      1021.8 | 5.48 ± 0.18 |

#### `copy_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                                                             |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------------------------------------------------|--------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |    68.1 ± 2.2 |     63.1 |     70.9 |      59.5 |       919.6 |         1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |   100.0 ± 1.4 |     98.5 |    103.4 |     217.1 |      1259.6 |  1.47 ± 0.05 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  127.8 ± 10.7 |    108.7 |    145.0 |     201.0 |      1309.7 |  1.88 ± 0.17 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |   343.7 ± 2.9 |    339.5 |    349.0 |     725.0 |       837.8 |  5.05 ± 0.17 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |   404.5 ± 4.4 |    397.0 |    410.3 |     225.3 |       473.7 |  5.94 ± 0.20 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |   654.8 ± 7.9 |    637.3 |    666.7 |      84.4 |       569.9 |  9.61 ± 0.33 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              |  776.3 ± 19.3 |    746.9 |    800.0 |     651.9 |       556.5 | 11.40 ± 0.46 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |   821.4 ± 8.8 |    808.5 |    841.4 |     105.3 |       714.1 | 12.06 ± 0.40 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 1689.6 ± 17.0 |   1660.3 |   1710.3 |     403.3 |      1264.4 | 24.81 ± 0.82 |

#### `copy_100_000_files_1G_bytes.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.143 ± 0.006 |   0.136 |   0.156 |    0.052 |      1.865 |         1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.172 ± 0.006 |   0.164 |   0.185 |    0.243 |      2.207 |  1.20 ± 0.06 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.184 ± 0.012 |   0.175 |   0.214 |    0.174 |      2.121 |  1.29 ± 0.10 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 0.343 ± 0.003 |   0.339 |   0.348 |    0.200 |      1.331 |  2.40 ± 0.10 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 0.969 ± 0.015 |   0.939 |   0.994 |    0.288 |      1.432 |  6.78 ± 0.29 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 1.122 ± 0.012 |   1.107 |   1.139 |    0.084 |      1.020 |  7.86 ± 0.32 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.150 ± 0.017 |   1.124 |   1.181 |    0.120 |      1.012 |  8.05 ± 0.34 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 1.213 ± 0.018 |   1.187 |   1.249 |    0.388 |      2.159 |  8.49 ± 0.36 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 2.070 ± 0.032 |   2.022 |   2.123 |    0.680 |      2.409 | 14.49 ± 0.61 |

#### `copy_100_000_files_1G_bytes_0_depth.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.234 ± 0.003 |   0.230 |   0.237 |    0.126 |      3.047 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.245 ± 0.002 |   0.241 |   0.248 |    0.136 |      2.969 | 1.05 ± 0.02 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 0.312 ± 0.006 |   0.308 |   0.329 |    0.161 |      1.209 | 1.33 ± 0.03 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.846 ± 0.009 |   0.833 |   0.857 |    0.030 |      0.808 | 3.61 ± 0.05 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 0.868 ± 0.014 |   0.853 |   0.903 |    0.257 |      1.271 | 3.71 ± 0.07 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 0.982 ± 0.007 |   0.972 |   0.992 |    0.072 |      0.893 | 4.20 ± 0.06 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.014 ± 0.014 |   0.992 |   1.036 |    0.091 |      0.906 | 4.33 ± 0.08 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 1.048 ± 0.014 |   1.029 |   1.074 |    0.333 |      1.906 | 4.48 ± 0.08 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 1.867 ± 0.038 |   1.812 |   1.943 |    0.559 |      2.137 | 7.98 ± 0.18 |

#### `copy_100_000_files_1G_bytes_0_depth_0_entropy.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.233 ± 0.003 |   0.230 |   0.237 |    0.127 |      3.037 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.248 ± 0.003 |   0.244 |   0.251 |    0.147 |      2.993 | 1.06 ± 0.02 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 0.308 ± 0.003 |   0.305 |   0.315 |    0.175 |      1.180 | 1.32 ± 0.02 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.852 ± 0.006 |   0.842 |   0.860 |    0.028 |      0.814 | 3.65 ± 0.05 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 0.863 ± 0.011 |   0.844 |   0.880 |    0.237 |      1.284 | 3.70 ± 0.07 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 0.986 ± 0.008 |   0.973 |   0.995 |    0.084 |      0.886 | 4.23 ± 0.06 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.001 ± 0.012 |   0.976 |   1.016 |    0.101 |      0.882 | 4.29 ± 0.07 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 1.047 ± 0.019 |   1.009 |   1.077 |    0.338 |      1.908 | 4.49 ± 0.10 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 1.876 ± 0.037 |   1.847 |   1.971 |    0.589 |      2.128 | 8.04 ± 0.19 |

#### `copy_100_000_files_1G_bytes_5_files_per_dir.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.145 ± 0.007 |   0.132 |   0.156 |    0.083 |      2.025 |         1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.164 ± 0.005 |   0.158 |   0.172 |    0.281 |      2.147 |  1.13 ± 0.06 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.178 ± 0.010 |   0.166 |   0.195 |    0.234 |      2.085 |  1.22 ± 0.09 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 0.349 ± 0.006 |   0.342 |   0.357 |    0.295 |      1.380 |  2.40 ± 0.12 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 0.972 ± 0.010 |   0.960 |   0.993 |    0.345 |      1.420 |  6.68 ± 0.31 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 1.183 ± 0.010 |   1.169 |   1.196 |    0.103 |      1.060 |  8.14 ± 0.37 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.318 ± 0.018 |   1.295 |   1.344 |    0.126 |      1.167 |  9.06 ± 0.43 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 1.369 ± 0.019 |   1.335 |   1.402 |    0.640 |      2.194 |  9.41 ± 0.45 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 1.952 ± 0.022 |   1.924 |   1.987 |    0.929 |      2.517 | 13.42 ± 0.63 |

#### `copy_10_000_files_0_bytes.md`

| Command                                                                             |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------------------------------------------------|-------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |    7.9 ± 0.6 |      7.2 |      9.8 |       5.0 |        81.6 |         1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |   12.1 ± 0.7 |     11.0 |     15.5 |      23.5 |       125.3 |  1.54 ± 0.14 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |   29.1 ± 2.1 |     27.3 |     42.1 |      52.1 |        82.1 |  3.71 ± 0.37 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  39.1 ± 10.3 |     23.8 |     71.9 |      16.9 |       114.1 |  4.97 ± 1.36 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |   40.4 ± 0.9 |     39.6 |     44.0 |      23.0 |        43.7 |  5.15 ± 0.38 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |   62.5 ± 0.9 |     61.0 |     64.1 |       8.2 |        53.6 |  7.95 ± 0.58 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |   72.8 ± 0.9 |     71.0 |     74.6 |       9.4 |        62.5 |  9.26 ± 0.67 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 156.8 ± 14.2 |    140.8 |    186.8 |     145.2 |        54.2 | 19.95 ± 2.30 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         |  202.5 ± 2.6 |    196.8 |    205.7 |      32.8 |       131.7 | 25.77 ± 1.87 |

#### `copy_10_000_files_1G_bytes.md`

| Command                                                                             |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------------------------------------------------|-------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |   84.2 ± 5.0 |     77.5 |     95.1 |      14.3 |      1116.4 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |   85.0 ± 4.6 |     78.2 |     96.6 |      52.7 |      1159.0 | 1.01 ± 0.08 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  112.0 ± 9.7 |     99.8 |    133.0 |      35.9 |      1114.1 | 1.33 ± 0.14 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  119.1 ± 3.9 |    112.9 |    125.1 |      27.0 |       469.7 | 1.41 ± 0.09 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 347.5 ± 11.5 |    333.9 |    363.9 |     148.0 |       663.0 | 4.13 ± 0.28 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  375.6 ± 4.5 |    368.3 |    382.2 |      14.3 |       359.0 | 4.46 ± 0.27 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  393.6 ± 3.9 |    387.4 |    399.4 |      23.5 |       367.7 | 4.67 ± 0.28 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  431.1 ± 4.2 |    424.2 |    440.6 |      63.6 |       713.3 | 5.12 ± 0.30 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 746.5 ± 14.7 |    724.6 |    768.7 |     339.8 |       836.0 | 8.86 ± 0.55 |

#### `copy_10_files_0_bytes.md`

| Command                                                                             |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  0.8 ± 0.0 |      0.7 |      1.3 |       0.6 |         0.1 |         1.00 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  1.0 ± 0.1 |      0.9 |      1.5 |       1.3 |         0.3 |  1.28 ± 0.13 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  1.4 ± 0.1 |      1.0 |      2.0 |       1.0 |         0.4 |  1.79 ± 0.14 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  1.5 ± 0.2 |      1.2 |      3.3 |       3.8 |         1.4 |  1.84 ± 0.23 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  1.8 ± 0.1 |      1.6 |      4.3 |       3.1 |         1.8 |  2.28 ± 0.23 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  1.8 ± 0.1 |      1.6 |      2.4 |       1.5 |         0.8 |  2.29 ± 0.20 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  2.5 ± 0.1 |      2.3 |      4.2 |       3.0 |         0.7 |  3.18 ± 0.26 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 13.2 ± 0.3 |     12.6 |     14.4 |      14.0 |        11.0 | 16.77 ± 1.10 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 48.6 ± 3.1 |     44.7 |     55.9 |       5.7 |         6.3 | 61.72 ± 5.43 |

#### `copy_10_files_1G_bytes.md`

| Command                                                                             |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------------------------------------------------|-------------:|---------:|---------:|----------:|------------:|------------:|
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |   66.1 ± 4.8 |     57.9 |     78.3 |       2.1 |       299.6 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |   66.8 ± 4.7 |     59.0 |     79.0 |       3.3 |       303.7 | 1.01 ± 0.10 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |   73.4 ± 4.8 |     68.8 |     87.2 |       0.7 |       228.7 | 1.11 ± 0.11 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |   99.7 ± 2.4 |     95.5 |    104.7 |       1.1 |       204.6 | 1.51 ± 0.12 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  188.2 ± 2.5 |    182.8 |    193.1 |       0.0 |       188.0 | 2.85 ± 0.21 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  189.8 ± 1.8 |    187.3 |    192.4 |       1.7 |       188.0 | 2.87 ± 0.21 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  231.4 ± 4.7 |    223.5 |    242.0 |      21.5 |       400.4 | 3.50 ± 0.26 |
| `./wcp/build/wcp /tmp/ftzz /tmp/ftzzz`                                              | 318.2 ± 21.5 |    309.0 |    379.1 |     106.2 |       385.2 | 4.81 ± 0.48 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 417.7 ± 24.5 |    398.3 |    465.2 |     171.8 |       385.0 | 6.32 ± 0.59 |
