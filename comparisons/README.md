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

## Remove

### Run

```bash
for num_bytes in {0,100M}; do
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

| Command                                        |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  5.252 ± 0.462 |   4.904 |   6.123 |    0.655 |     23.224 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  6.755 ± 0.585 |   6.046 |   7.871 |    1.462 |     24.011 | 1.29 ± 0.16 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 11.622 ± 0.072 |  11.490 |  11.725 |    0.479 |     10.856 | 2.21 ± 0.19 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 11.643 ± 0.058 |  11.560 |  11.739 |    0.475 |     10.878 | 2.22 ± 0.20 |
| `find /tmp/ftzz -delete`                       | 11.870 ± 0.079 |  11.782 |  12.039 |    0.617 |     10.923 | 2.26 ± 0.20 |
| `rm -r /tmp/ftzz`                              | 11.873 ± 0.029 |  11.837 |  11.923 |    0.580 |     10.995 | 2.26 ± 0.20 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 13.733 ± 0.056 |  13.658 |  13.825 |    1.133 |     12.339 | 2.61 ± 0.23 |

#### `remove_1M_files_100M_bytes.md`

| Command                                        |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  7.953 ± 0.686 |   7.146 |   9.129 |    0.675 |     37.039 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  9.168 ± 0.498 |   8.457 |  10.061 |    1.579 |     38.218 | 1.15 ± 0.12 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 23.807 ± 0.310 |  23.279 |  24.275 |    1.232 |     20.423 | 2.99 ± 0.26 |
| `rm -r /tmp/ftzz`                              | 27.295 ± 0.987 |  24.878 |  28.402 |    0.678 |     22.116 | 3.43 ± 0.32 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 27.312 ± 1.177 |  25.140 |  29.140 |    0.561 |     22.185 | 3.43 ± 0.33 |
| `find /tmp/ftzz -delete`                       | 27.709 ± 0.573 |  26.476 |  28.333 |    0.695 |     22.336 | 3.48 ± 0.31 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 27.991 ± 1.319 |  25.605 |  29.585 |    0.568 |     22.156 | 3.52 ± 0.35 |

#### `remove_100_000_files_0_bytes.md`

| Command                                        |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 548.5 ± 159.5 |    352.1 |    859.8 |      65.0 |      1852.8 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  661.5 ± 97.1 |    542.2 |    864.4 |     166.1 |      2184.0 | 1.21 ± 0.39 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  822.7 ± 23.7 |    792.4 |    868.6 |      39.1 |       737.2 | 1.50 ± 0.44 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  826.9 ± 26.3 |    785.6 |    872.3 |      41.0 |       735.4 | 1.51 ± 0.44 |
| `rm -r /tmp/ftzz`                              |  843.2 ± 10.6 |    820.5 |    861.8 |      49.3 |       742.6 | 1.54 ± 0.45 |
| `find /tmp/ftzz -delete`                       |  848.8 ± 18.4 |    819.8 |    878.7 |      52.5 |       735.4 | 1.55 ± 0.45 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1005.7 ± 12.7 |    993.1 |   1035.2 |      97.8 |       843.5 | 1.83 ± 0.53 |

#### `remove_100_000_files_0_bytes_0_depth.md`

| Command                                        |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  939.5 ± 12.5 |    930.4 |    973.3 |      36.4 |       882.4 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             |  941.3 ± 19.9 |    925.9 |    993.1 |      34.6 |       889.1 | 1.00 ± 0.02 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  950.2 ± 18.3 |    930.6 |    985.8 |      39.5 |       884.2 | 1.01 ± 0.02 |
| `rm -r /tmp/ftzz`                              |  969.0 ± 14.9 |    939.1 |    991.3 |      62.9 |       885.3 | 1.03 ± 0.02 |
| `find /tmp/ftzz -delete`                       |  976.9 ± 13.9 |    958.5 |   1011.3 |      66.7 |       888.8 | 1.04 ± 0.02 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      |  1087.9 ± 9.6 |   1071.5 |   1102.6 |     100.0 |       924.2 | 1.16 ± 0.02 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`         |  1209.8 ± 6.2 |   1200.4 |   1217.7 |     145.5 |      1047.3 | 1.29 ± 0.02 |
| `./target/release/rm_rayon /tmp/ftzz`          | 1273.4 ± 14.4 |   1252.0 |   1297.3 |     100.0 |      2192.6 | 1.36 ± 0.02 |

#### `remove_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.814 ± 0.204 |   0.548 |   1.139 |    0.165 |      2.931 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 1.004 ± 0.099 |   0.853 |   1.171 |    0.384 |      3.453 | 1.23 ± 0.33 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.195 ± 0.013 |   1.173 |   1.208 |    0.109 |      1.034 | 1.47 ± 0.37 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.212 ± 0.024 |   1.181 |   1.267 |    0.110 |      1.036 | 1.49 ± 0.37 |
| `find /tmp/ftzz -delete`                       | 1.372 ± 0.040 |   1.327 |   1.454 |    0.177 |      1.130 | 1.69 ± 0.42 |
| `rm -r /tmp/ftzz`                              | 1.403 ± 0.029 |   1.367 |   1.445 |    0.190 |      1.154 | 1.72 ± 0.43 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.507 ± 0.026 |   1.462 |   1.565 |    0.193 |      1.224 | 1.85 ± 0.46 |

#### `remove_100_000_files_100M_bytes.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.541 ± 0.138 |   0.363 |   0.720 |    0.062 |      2.298 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.672 ± 0.196 |   0.449 |   0.993 |    0.153 |      2.507 | 1.24 ± 0.48 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.013 ± 0.007 |   1.001 |   1.024 |    0.045 |      0.939 | 1.87 ± 0.48 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.014 ± 0.009 |   1.004 |   1.036 |    0.044 |      0.941 | 1.87 ± 0.48 |
| `find /tmp/ftzz -delete`                       | 1.031 ± 0.008 |   1.019 |   1.047 |    0.055 |      0.949 | 1.91 ± 0.48 |
| `rm -r /tmp/ftzz`                              | 1.048 ± 0.042 |   1.015 |   1.163 |    0.048 |      0.965 | 1.94 ± 0.50 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.224 ± 0.017 |   1.204 |   1.261 |    0.101 |      1.057 | 2.26 ± 0.58 |

#### `remove_100_000_files_100M_bytes_0_depth.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 1.155 ± 0.010 |   1.143 |   1.172 |    0.032 |      1.098 |        1.00 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.162 ± 0.010 |   1.150 |   1.182 |    0.042 |      1.095 | 1.01 ± 0.01 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.166 ± 0.014 |   1.150 |   1.194 |    0.041 |      1.096 | 1.01 ± 0.01 |
| `rm -r /tmp/ftzz`                              | 1.190 ± 0.007 |   1.179 |   1.205 |    0.062 |      1.107 | 1.03 ± 0.01 |
| `find /tmp/ftzz -delete`                       | 1.192 ± 0.005 |   1.184 |   1.200 |    0.063 |      1.109 | 1.03 ± 0.01 |
| `./target/release/rm_rayon /tmp/ftzz`          | 1.246 ± 0.010 |   1.234 |   1.269 |    0.112 |      2.736 | 1.08 ± 0.01 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.300 ± 0.008 |   1.286 |   1.311 |    0.100 |      1.148 | 1.13 ± 0.01 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`         | 1.434 ± 0.006 |   1.426 |   1.447 |    0.146 |      1.272 | 1.24 ± 0.01 |

#### `remove_100_000_files_100M_bytes_5_files_per_dir.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.649 ± 0.063 |   0.593 |   0.769 |    0.134 |      2.922 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.912 ± 0.143 |   0.749 |   1.165 |    0.324 |      3.599 | 1.41 ± 0.26 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.444 ± 0.032 |   1.412 |   1.517 |    0.111 |      1.279 | 2.23 ± 0.22 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.453 ± 0.020 |   1.430 |   1.500 |    0.109 |      1.284 | 2.24 ± 0.22 |
| `find /tmp/ftzz -delete`                       | 1.593 ± 0.024 |   1.573 |   1.656 |    0.189 |      1.351 | 2.45 ± 0.24 |
| `rm -r /tmp/ftzz`                              | 1.636 ± 0.015 |   1.612 |   1.665 |    0.187 |      1.392 | 2.52 ± 0.25 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.754 ± 0.024 |   1.730 |   1.798 |    0.198 |      1.479 | 2.70 ± 0.26 |

#### `remove_10_000_files_0_bytes.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  45.0 ± 1.6 |     43.4 |     48.1 |      11.3 |       210.8 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  55.4 ± 1.9 |     52.5 |     59.3 |      24.3 |       223.5 | 1.23 ± 0.06 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 108.4 ± 0.8 |    106.4 |    109.5 |       8.8 |        97.3 | 2.41 ± 0.09 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 109.5 ± 4.0 |    106.5 |    126.0 |       9.6 |        96.9 | 2.43 ± 0.12 |
| `find /tmp/ftzz -delete`                       | 119.6 ± 2.2 |    117.2 |    124.1 |      15.4 |       101.4 | 2.66 ± 0.11 |
| `rm -r /tmp/ftzz`                              | 120.1 ± 0.9 |    118.7 |    121.7 |      15.7 |       101.8 | 2.67 ± 0.10 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 177.7 ± 0.7 |    176.4 |    179.1 |      18.2 |       118.3 | 3.95 ± 0.14 |

#### `remove_10_000_files_100M_bytes.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  51.8 ± 1.6 |     49.2 |     54.6 |      10.4 |       288.9 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  62.8 ± 2.6 |     59.7 |     70.4 |      24.1 |       306.4 | 1.21 ± 0.06 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 154.2 ± 0.7 |    152.8 |    155.4 |       8.8 |       141.0 | 2.98 ± 0.09 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 154.3 ± 1.3 |    152.3 |    156.7 |      10.1 |       139.7 | 2.98 ± 0.09 |
| `find /tmp/ftzz -delete`                       | 164.7 ± 1.2 |    161.6 |    165.9 |      13.9 |       145.1 | 3.18 ± 0.10 |
| `rm -r /tmp/ftzz`                              | 165.9 ± 0.8 |    164.4 |    166.8 |      16.5 |       144.7 | 3.20 ± 0.10 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 226.1 ± 5.2 |    223.5 |    240.7 |      20.0 |       161.8 | 4.36 ± 0.17 |

#### `remove_10_files_0_bytes.md`

| Command                                        |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  1.1 ± 0.0 |      1.0 |      1.2 |       0.3 |         0.7 |         1.00 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  1.1 ± 0.0 |      1.0 |      1.2 |       0.2 |         0.8 |  1.00 ± 0.03 |
| *`./target/release/rmz /tmp/ftzz`*             |  1.1 ± 0.1 |      1.0 |      1.4 |       0.6 |         1.4 |  1.01 ± 0.06 |
| `./target/release/rm_rayon /tmp/ftzz`          |  1.4 ± 0.1 |      1.2 |      1.6 |       1.1 |         2.6 |  1.30 ± 0.07 |
| `rm -r /tmp/ftzz`                              |  3.6 ± 0.1 |      3.4 |      4.3 |       1.0 |         2.4 |  3.33 ± 0.11 |
| `find /tmp/ftzz -delete`                       |  4.2 ± 0.1 |      3.9 |      5.6 |       1.1 |         2.5 |  3.94 ± 0.16 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 46.8 ± 0.3 |     45.1 |     47.1 |       2.2 |         5.3 | 43.90 ± 1.07 |

#### `remove_10_files_100M_bytes.md`

| Command                                        |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|-----------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rm_rayon /tmp/ftzz`          |  8.2 ± 0.2 |      7.8 |      9.0 |       1.3 |        49.6 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             |  9.0 ± 0.3 |      8.4 |      9.6 |       0.6 |        30.7 | 1.10 ± 0.05 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 25.1 ± 0.2 |     24.6 |     26.2 |       0.2 |        24.6 | 3.05 ± 0.10 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 25.1 ± 0.1 |     24.8 |     25.4 |       0.3 |        24.5 | 3.05 ± 0.09 |
| `rm -r /tmp/ftzz`                              | 25.6 ± 1.2 |     25.1 |     33.9 |       0.5 |        24.7 | 3.11 ± 0.17 |
| `find /tmp/ftzz -delete`                       | 26.3 ± 0.7 |     25.9 |     31.4 |       0.5 |        24.9 | 3.19 ± 0.13 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 75.8 ± 3.1 |     69.9 |     82.0 |       1.8 |        34.1 | 9.20 ± 0.47 |

## Copy

### Setup

```bash
cargo install fcp xcp
git clone https://github.com/wheybags/wcp.git && mkdir wcp/build && (cd wcp/build && cmake .. -DCMAKE_BUILD_TYPE=Release && make)
```

### Run

```bash
for num_bytes in {0,100M}; do
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
  --export-markdown "benches/copy_100_000_files_100M_bytes_0_depth_0_entropy.md" \
  --export-json "benches/copy_100_000_files_100M_bytes_0_depth_0_entropy.json" \
  --setup "ftzz g -n 100_000 -b 100M -d 0 --fill-byte 0 /tmp/ftzz" \
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

> Note: my benchmarking machine doesn't have io_uring enabled, so I was not able to include results
> with `wcp`. That said, I ran a few quick benches on my personal machine against `cpz` and found it
> to be faster than `wcp`.

#### `copy_10_files_0_bytes.md`

| Command                                                                                            |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:---------------------------------------------------------------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |  1.9 ± 0.1 |      1.7 |      2.4 |       0.8 |         2.6 |         1.00 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |  2.5 ± 0.3 |      2.1 |      3.1 |       0.6 |         1.7 |  1.33 ± 0.17 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |  3.0 ± 0.3 |      2.0 |      3.7 |       2.6 |         6.5 |  1.61 ± 0.20 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |  3.7 ± 0.2 |      3.2 |      4.3 |       3.3 |         7.4 |  1.96 ± 0.16 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       |  3.9 ± 0.4 |      3.2 |      4.4 |       1.1 |         2.7 |  2.09 ± 0.24 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |  3.9 ± 0.2 |      3.6 |      4.8 |       2.5 |         4.9 |  2.09 ± 0.17 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> |  7.8 ± 1.4 |      6.9 |     10.8 |       5.1 |         6.5 |  4.14 ± 0.79 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 48.6 ± 0.2 |     48.1 |     49.0 |       2.9 |         6.9 | 25.82 ± 1.43 |

#### `copy_10_000_files_0_bytes.md`

| Command                                                                                            |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:---------------------------------------------------------------------------------------------------|-------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |   48.6 ± 1.1 |     46.8 |     50.6 |      42.7 |       263.6 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |   63.9 ± 1.0 |     61.8 |     65.6 |      83.5 |       344.2 | 1.32 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |   66.4 ± 1.0 |     64.0 |     68.0 |      92.3 |       353.3 | 1.37 ± 0.04 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |  110.8 ± 4.8 |    103.4 |    120.5 |      84.4 |       392.9 | 2.28 ± 0.11 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> |  174.4 ± 1.7 |    171.9 |    176.9 |      59.1 |       175.9 | 3.59 ± 0.09 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |  239.7 ± 1.4 |    238.0 |    242.6 |      48.9 |       185.3 | 4.94 ± 0.12 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 265.9 ± 38.7 |    227.4 |    311.6 |      45.5 |       188.8 | 5.48 ± 0.81 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        |  446.4 ± 3.4 |    442.4 |    452.2 |     108.2 |       298.2 | 9.19 ± 0.23 |

#### `copy_100_000_files_0_bytes.md`

| Command                                                                                            |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      | 0.357 ± 0.009 |   0.347 |   0.374 |    0.266 |      1.976 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   | 0.467 ± 0.005 |   0.462 |   0.476 |    0.525 |      2.607 | 1.31 ± 0.03 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         | 0.716 ± 0.010 |   0.705 |   0.734 |    0.626 |      4.207 | 2.01 ± 0.06 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      | 0.971 ± 0.017 |   0.947 |   0.992 |    0.552 |      3.389 | 2.72 ± 0.08 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 1.339 ± 0.023 |   1.302 |   1.372 |    0.401 |      1.298 | 3.75 ± 0.11 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 1.672 ± 0.015 |   1.651 |   1.699 |    0.285 |      1.355 | 4.68 ± 0.12 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  | 1.772 ± 0.007 |   1.763 |   1.784 |    0.311 |      1.428 | 4.97 ± 0.12 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 2.897 ± 0.022 |   2.870 |   2.922 |    0.665 |      2.174 | 8.12 ± 0.21 |

#### `copy_1M_files_0_bytes.md`

| Command                                                                                            |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |  5.285 ± 0.513 |   4.963 |   6.377 |    3.209 |     28.408 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |  7.089 ± 0.579 |   6.502 |   8.005 |    6.581 |     37.199 | 1.34 ± 0.17 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      | 14.600 ± 0.128 |  14.508 |  14.944 |    6.550 |     48.207 | 2.76 ± 0.27 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         | 14.997 ± 0.079 |  14.877 |  15.121 |    6.559 |     75.957 | 2.84 ± 0.28 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 18.659 ± 0.155 |  18.467 |  18.940 |    4.751 |     18.415 | 3.53 ± 0.34 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 22.796 ± 0.103 |  22.547 |  22.909 |    3.457 |     18.937 | 4.31 ± 0.42 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  | 23.958 ± 0.119 |  23.787 |  24.198 |    3.885 |     19.659 | 4.53 ± 0.44 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 34.984 ± 0.132 |  34.806 |  35.188 |    7.984 |     28.287 | 6.62 ± 0.64 |

#### `copy_10_files_100M_bytes.md`

| Command                                                                                            |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |       Relative |
|:---------------------------------------------------------------------------------------------------|------------:|---------:|---------:|----------:|------------:|---------------:|
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |   1.6 ± 0.3 |      1.3 |      4.0 |       1.3 |         3.5 |           1.00 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       |   1.9 ± 0.3 |      1.8 |      9.7 |       0.5 |         1.3 |    1.21 ± 0.29 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |   2.1 ± 0.3 |      1.8 |      9.5 |       0.9 |         3.1 |    1.32 ± 0.32 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |   2.9 ± 0.5 |      2.2 |      4.2 |       2.6 |         6.3 |    1.83 ± 0.46 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |   3.5 ± 0.3 |      3.1 |      5.1 |       2.3 |         4.9 |    2.23 ± 0.48 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |   3.7 ± 0.1 |      3.5 |      4.0 |       0.8 |         2.7 |    2.39 ± 0.48 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 145.8 ± 0.9 |    144.1 |    147.3 |      40.6 |       216.7 |  93.50 ± 18.40 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 518.0 ± 1.6 |    516.4 |    521.4 |     713.0 |       174.8 | 332.20 ± 65.37 |

#### `copy_10_000_files_100M_bytes.md`

| Command                                                                                            |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:---------------------------------------------------------------------------------------------------|--------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |    75.0 ± 2.0 |     72.0 |     78.1 |      48.4 |       453.3 |         1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |    91.6 ± 2.3 |     88.2 |     96.0 |      82.0 |       534.9 |  1.22 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |    97.1 ± 2.9 |     93.2 |    102.3 |      97.4 |       552.0 |  1.29 ± 0.05 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |   139.9 ± 6.0 |    130.4 |    151.6 |      90.7 |       495.1 |  1.87 ± 0.09 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       |   326.2 ± 2.5 |    322.5 |    329.6 |      48.2 |       269.5 |  4.35 ± 0.12 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |   338.4 ± 2.1 |    334.8 |    341.3 |      48.6 |       281.8 |  4.51 ± 0.12 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> |  340.6 ± 19.7 |    329.8 |    395.6 |     112.5 |       402.9 |  4.54 ± 0.29 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 1488.0 ± 41.8 |   1422.8 |   1552.6 |     619.6 |       892.3 | 19.84 ± 0.76 |

#### `copy_100_000_files_100M_bytes.md`

| Command                                                                                            |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:---------------------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |  0.751 ± 0.136 |   0.617 |   1.068 |    0.318 |      3.768 |         1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |  0.828 ± 0.074 |   0.752 |   1.003 |    0.550 |      4.302 |  1.10 ± 0.22 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |  0.958 ± 0.030 |   0.931 |   1.025 |    0.709 |      5.300 |  1.28 ± 0.23 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |  1.135 ± 0.013 |   1.113 |   1.151 |    0.616 |      3.997 |  1.51 ± 0.27 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> |  1.835 ± 0.047 |   1.792 |   1.942 |    0.558 |      1.960 |  2.44 ± 0.45 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       |  2.524 ± 0.021 |   2.486 |   2.549 |    0.295 |      2.170 |  3.36 ± 0.61 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |  2.610 ± 0.022 |   2.571 |   2.638 |    0.330 |      2.221 |  3.48 ± 0.63 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 10.099 ± 0.161 |   9.891 |  10.500 |    1.277 |      6.656 | 13.45 ± 2.45 |

#### `copy_1M_files_100M_bytes.md`

| Command                                                                                            |        Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:---------------------------------------------------------------------------------------------------|----------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |   8.840 ± 0.210 |   8.488 |   9.176 |    3.759 |     47.928 |         1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |  10.485 ± 0.179 |  10.175 |  10.744 |    6.621 |     55.259 |  1.19 ± 0.03 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |  16.481 ± 0.156 |  16.294 |  16.735 |    7.566 |     57.508 |  1.86 ± 0.05 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |  16.924 ± 0.103 |  16.805 |  17.172 |    7.077 |     85.419 |  1.91 ± 0.05 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> |  29.514 ± 3.214 |  27.382 |  36.505 |    7.532 |     28.394 |  3.34 ± 0.37 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       |  35.050 ± 0.149 |  34.878 |  35.297 |    3.618 |     30.658 |  3.97 ± 0.10 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |  35.533 ± 0.135 |  35.257 |  35.723 |    4.079 |     30.679 |  4.02 ± 0.10 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 103.297 ± 1.387 | 102.166 | 106.951 |   11.423 |     63.319 | 11.69 ± 0.32 |

#### `copy_100_000_files_0_bytes_0_depth.md`

| Command                                                                                            |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      | 1.266 ± 0.021 |   1.245 |   1.320 |    0.521 |      4.012 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         | 1.420 ± 0.011 |   1.408 |   1.438 |    0.456 |      7.234 | 1.12 ± 0.02 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   | 1.434 ± 0.004 |   1.429 |   1.439 |    0.431 |      7.467 | 1.13 ± 0.02 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      | 1.558 ± 0.013 |   1.542 |   1.582 |    0.188 |      1.336 | 1.23 ± 0.02 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 1.595 ± 0.018 |   1.571 |   1.636 |    0.395 |      1.563 | 1.26 ± 0.03 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 1.845 ± 0.009 |   1.833 |   1.866 |    0.285 |      1.527 | 1.46 ± 0.03 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  | 1.954 ± 0.007 |   1.944 |   1.963 |    0.311 |      1.609 | 1.54 ± 0.03 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 2.419 ± 0.025 |   2.375 |   2.466 |    0.647 |      2.316 | 1.91 ± 0.04 |

#### `copy_100_000_files_100M_bytes_0_depth.md`

| Command                                                                                            |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      | 1.363 ± 0.012 |   1.349 |   1.383 |    0.591 |      4.809 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   | 1.510 ± 0.014 |   1.495 |   1.543 |    0.455 |      7.824 | 1.11 ± 0.01 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         | 1.523 ± 0.013 |   1.503 |   1.542 |    0.498 |      7.600 | 1.12 ± 0.01 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 2.090 ± 0.032 |   2.047 |   2.136 |    0.555 |      2.218 | 1.53 ± 0.03 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      | 2.361 ± 0.018 |   2.340 |   2.403 |    0.231 |      2.067 | 1.73 ± 0.02 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  | 2.662 ± 0.019 |   2.635 |   2.703 |    0.328 |      2.269 | 1.95 ± 0.02 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 2.770 ± 0.062 |   2.728 |   2.910 |    0.292 |      2.401 | 2.03 ± 0.05 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 8.267 ± 0.145 |   8.117 |   8.606 |    1.204 |      5.029 | 6.07 ± 0.12 |

#### `copy_100_000_files_100M_bytes_0_depth_0_entropy.md`

| Command                                                                                            |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      | 1.362 ± 0.016 |   1.342 |   1.398 |    0.599 |      4.780 |        1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   | 1.510 ± 0.011 |   1.489 |   1.525 |    0.461 |      7.893 | 1.11 ± 0.02 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         | 1.557 ± 0.058 |   1.516 |   1.717 |    0.504 |      7.664 | 1.14 ± 0.05 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 2.075 ± 0.021 |   2.048 |   2.119 |    0.547 |      2.211 | 1.52 ± 0.02 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      | 2.385 ± 0.015 |   2.363 |   2.411 |    0.233 |      2.093 | 1.75 ± 0.02 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 2.570 ± 0.014 |   2.552 |   2.591 |    0.292 |      2.218 | 1.89 ± 0.02 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  | 2.661 ± 0.017 |   2.628 |   2.693 |    0.324 |      2.276 | 1.95 ± 0.03 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 8.242 ± 0.131 |   8.085 |   8.496 |    1.215 |      4.989 | 6.05 ± 0.12 |

#### `copy_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                                                                            |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:---------------------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         | 0.627 ± 0.009 |   0.617 |   0.643 |    0.852 |      3.414 |        1.00 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      | 0.646 ± 0.039 |   0.587 |   0.723 |    0.424 |      2.935 | 1.03 ± 0.06 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   | 0.647 ± 0.054 |   0.600 |   0.753 |    0.753 |      3.421 | 1.03 ± 0.09 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      | 1.005 ± 0.030 |   0.952 |   1.048 |    0.940 |      3.642 | 1.60 ± 0.05 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> | 1.727 ± 0.025 |   1.703 |   1.777 |    0.585 |      1.757 | 2.75 ± 0.06 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       | 2.243 ± 0.020 |   2.206 |   2.263 |    0.396 |      1.805 | 3.58 ± 0.06 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  | 2.301 ± 0.009 |   2.283 |   2.310 |    0.432 |      1.825 | 3.67 ± 0.05 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 3.865 ± 0.031 |   3.808 |   3.901 |    0.988 |      2.847 | 6.16 ± 0.10 |

#### `copy_100_000_files_100M_bytes_5_files_per_dir.md`

| Command                                                                                            |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:---------------------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                                      |  1.031 ± 0.065 |   0.924 |   1.115 |    0.495 |      4.988 |         1.00 |
| `./target/release/rayon_cp /tmp/ftzz /tmp/ftzzz`                                                   |  1.041 ± 0.080 |   0.953 |   1.175 |    0.788 |      5.488 |  1.01 ± 0.10 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                                         |  1.080 ± 0.132 |   0.976 |   1.361 |    0.902 |      5.550 |  1.05 ± 0.14 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                                      |  1.327 ± 0.037 |   1.296 |   1.410 |    0.872 |      4.829 |  1.29 ± 0.09 |
| <code>sh -c '(cd /tmp/ftzz; tar cf - .) &#124; (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'</code> |  2.311 ± 0.043 |   2.249 |   2.407 |    0.801 |      2.524 |  2.24 ± 0.15 |
| `./target/release/stdlib_cp /tmp/ftzz /tmp/ftzzz`                                                  |  3.255 ± 0.022 |   3.230 |   3.287 |    0.455 |      2.718 |  3.16 ± 0.20 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                                       |  3.309 ± 0.020 |   3.281 |   3.338 |    0.418 |      2.806 |  3.21 ± 0.20 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                                        | 11.038 ± 0.206 |  10.806 |  11.411 |    1.842 |      7.375 | 10.70 ± 0.71 |
