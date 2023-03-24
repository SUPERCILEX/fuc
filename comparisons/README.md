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

- Linux: 5.14.0-162.18.1.el9_1.x86_64
- CPU: Intel i7-7700
- Memory: 15GiB
- `/dev/sda6 /tmp xfs rw,nosuid,nodev,relatime,attr2,inode64,logbufs=8,logbsize=32k,noquota 0 0`

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

| Command                                        |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  4.711 ± 0.649 |   4.208 |   6.099 |    0.535 |     18.650 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  5.689 ± 0.549 |   5.007 |   6.630 |    1.253 |     20.088 | 1.21 ± 0.20 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  9.345 ± 0.063 |   9.282 |   9.457 |    0.377 |      8.708 | 1.98 ± 0.27 |
| `rm -r /tmp/ftzz`                              |  9.491 ± 0.059 |   9.444 |   9.650 |    0.465 |      8.768 | 2.01 ± 0.28 |
| `find /tmp/ftzz -delete`                       |  9.535 ± 0.051 |   9.434 |   9.620 |    0.485 |      8.804 | 2.02 ± 0.28 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 11.046 ± 0.049 |  10.989 |  11.120 |    0.911 |      9.914 | 2.34 ± 0.32 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 12.647 ± 0.088 |  12.532 |  12.758 |    1.376 |     10.950 | 2.68 ± 0.37 |

#### `remove_1M_files_1G_bytes.md`

| Command                                        |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  6.559 ± 0.463 |   5.977 |   7.689 |    0.556 |     28.872 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  7.765 ± 0.454 |   7.047 |   8.567 |    1.331 |     30.759 | 1.18 ± 0.11 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 18.924 ± 0.321 |  18.394 |  19.428 |    0.977 |     16.014 | 2.89 ± 0.21 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 22.892 ± 0.885 |  21.842 |  24.252 |    0.451 |     17.758 | 3.49 ± 0.28 |
| `find /tmp/ftzz -delete`                       | 23.020 ± 0.755 |  21.342 |  23.911 |    0.601 |     17.805 | 3.51 ± 0.27 |
| `rm -r /tmp/ftzz`                              | 23.354 ± 0.990 |  21.777 |  24.772 |    0.563 |     17.907 | 3.56 ± 0.29 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 26.357 ± 0.758 |  24.981 |  27.640 |    1.505 |     20.445 | 4.02 ± 0.31 |

#### `remove_100_000_files_0_bytes.md`

| Command                                        |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|--------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  459.3 ± 96.0 |    369.7 |    583.3 |      65.2 |      1697.5 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  648.1 ± 94.7 |    535.5 |    811.3 |     158.6 |      2121.1 | 1.41 ± 0.36 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  804.2 ± 19.5 |    777.9 |    848.2 |      42.5 |       723.3 | 1.75 ± 0.37 |
| `find /tmp/ftzz -delete`                       |  826.2 ± 22.6 |    796.3 |    853.0 |      55.3 |       728.1 | 1.80 ± 0.38 |
| `rm -r /tmp/ftzz`                              |  873.7 ± 54.2 |    813.7 |    968.5 |      53.2 |       744.3 | 1.90 ± 0.41 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1012.7 ± 16.8 |    996.3 |   1045.1 |      96.1 |       846.1 | 2.21 ± 0.46 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1132.0 ± 46.9 |   1099.2 |   1243.2 |     136.3 |       945.9 | 2.46 ± 0.53 |

#### `remove_100_000_files_0_bytes_0_depth.md`

| Command                                        |      Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|---------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |   830.7 ± 21.7 |    805.1 |    863.2 |      31.6 |       769.5 |        1.00 |
| `./target/release/rm_stdlib /tmp/ftzz`         |   833.8 ± 26.8 |    806.7 |    881.3 |      34.7 |       769.5 | 1.00 ± 0.04 |
| `rm -r /tmp/ftzz`                              |   847.1 ± 15.3 |    833.9 |    871.3 |      53.1 |       775.2 | 1.02 ± 0.03 |
| `find /tmp/ftzz -delete`                       |   849.7 ± 14.8 |    835.2 |    875.2 |      50.3 |       781.1 | 1.02 ± 0.03 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      |   978.3 ± 24.4 |    954.3 |   1033.6 |      89.4 |       819.5 | 1.18 ± 0.04 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`         |   1070.0 ± 5.3 |   1063.0 |   1078.6 |     132.8 |       923.4 | 1.29 ± 0.03 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  1155.9 ± 42.9 |   1104.6 |   1256.0 |     124.8 |       984.6 | 1.39 ± 0.06 |
| `./target/release/rm_rayon /tmp/ftzz`          | 1193.3 ± 307.0 |   1082.7 |   2066.6 |     102.5 |      2030.4 | 1.44 ± 0.37 |

#### `remove_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.640 ± 0.189 |   0.445 |   0.950 |    0.145 |      2.228 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.784 ± 0.110 |   0.670 |   0.961 |    0.303 |      2.610 | 1.22 ± 0.40 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 0.988 ± 0.057 |   0.940 |   1.117 |    0.091 |      0.832 | 1.54 ± 0.46 |
| `find /tmp/ftzz -delete`                       | 1.090 ± 0.021 |   1.056 |   1.136 |    0.164 |      0.883 | 1.70 ± 0.50 |
| `rm -r /tmp/ftzz`                              | 1.123 ± 0.013 |   1.098 |   1.141 |    0.153 |      0.921 | 1.75 ± 0.52 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.190 ± 0.020 |   1.168 |   1.238 |    0.168 |      0.956 | 1.86 ± 0.55 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.340 ± 0.031 |   1.304 |   1.405 |    0.224 |      1.045 | 2.09 ± 0.62 |

#### `remove_100_000_files_1G_bytes.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.558 ± 0.129 |   0.400 |   0.734 |    0.063 |      2.339 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.625 ± 0.116 |   0.535 |   0.934 |    0.141 |      2.450 | 1.12 ± 0.33 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.104 ± 0.014 |   1.080 |   1.132 |    0.043 |      1.020 | 1.98 ± 0.46 |
| `find /tmp/ftzz -delete`                       | 1.128 ± 0.016 |   1.110 |   1.155 |    0.054 |      1.027 | 2.02 ± 0.47 |
| `rm -r /tmp/ftzz`                              | 1.129 ± 0.008 |   1.117 |   1.142 |    0.054 |      1.035 | 2.02 ± 0.47 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.311 ± 0.005 |   1.301 |   1.317 |    0.098 |      1.138 | 2.35 ± 0.54 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.454 ± 0.014 |   1.434 |   1.478 |    0.143 |      1.261 | 2.60 ± 0.60 |

#### `remove_100_000_files_1G_bytes_0_depth.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `./target/release/rm_rayon /tmp/ftzz`          | 1.087 ± 0.005 |   1.081 |   1.097 |    0.092 |      2.516 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             | 1.115 ± 0.025 |   1.090 |   1.173 |    0.028 |      1.048 | 1.03 ± 0.02 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.124 ± 0.038 |   1.094 |   1.208 |    0.034 |      1.044 | 1.03 ± 0.04 |
| `rm -r /tmp/ftzz`                              | 1.129 ± 0.017 |   1.102 |   1.158 |    0.052 |      1.046 | 1.04 ± 0.02 |
| `find /tmp/ftzz -delete`                       | 1.137 ± 0.011 |   1.120 |   1.156 |    0.057 |      1.048 | 1.05 ± 0.01 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.251 ± 0.019 |   1.235 |   1.301 |    0.087 |      1.093 | 1.15 ± 0.02 |
| `perl -e 'for(</tmp/ftzz/*>){unlink}'`         | 1.354 ± 0.008 |   1.342 |   1.367 |    0.130 |      1.192 | 1.24 ± 0.01 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.470 ± 0.031 |   1.438 |   1.537 |    0.127 |      1.285 | 1.35 ± 0.03 |

#### `remove_100_000_files_1G_bytes_5_files_per_dir.md`

| Command                                        |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:-----------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             | 0.508 ± 0.034 |   0.483 |   0.600 |    0.115 |      2.352 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          | 0.683 ± 0.059 |   0.599 |   0.779 |    0.261 |      2.828 | 1.34 ± 0.15 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 1.246 ± 0.033 |   1.209 |   1.327 |    0.102 |      1.082 | 2.45 ± 0.18 |
| `find /tmp/ftzz -delete`                       | 1.387 ± 0.034 |   1.354 |   1.462 |    0.169 |      1.159 | 2.73 ± 0.19 |
| `rm -r /tmp/ftzz`                              | 1.402 ± 0.023 |   1.372 |   1.451 |    0.163 |      1.182 | 2.76 ± 0.19 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 1.485 ± 0.012 |   1.460 |   1.501 |    0.168 |      1.237 | 2.92 ± 0.20 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 1.632 ± 0.026 |   1.598 |   1.689 |    0.230 |      1.338 | 3.21 ± 0.22 |

#### `remove_10_000_files_0_bytes.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  33.9 ± 1.1 |     32.1 |     36.3 |       9.2 |       145.0 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  40.1 ± 2.3 |     37.2 |     48.1 |      18.6 |       160.0 | 1.18 ± 0.08 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  77.2 ± 1.3 |     75.3 |     80.0 |       7.2 |        68.4 | 2.28 ± 0.08 |
| `find /tmp/ftzz -delete`                       |  85.4 ± 1.3 |     84.0 |     88.5 |      10.1 |        73.0 | 2.52 ± 0.09 |
| `rm -r /tmp/ftzz`                              |  86.1 ± 1.5 |     84.8 |     89.2 |      11.3 |        73.0 | 2.54 ± 0.09 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 110.8 ± 1.5 |    109.1 |    113.8 |      18.2 |        90.2 | 3.27 ± 0.12 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 138.6 ± 1.3 |    136.8 |    143.0 |      14.1 |        83.7 | 4.09 ± 0.14 |

#### `remove_10_000_files_1G_bytes.md`

| Command                                        |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  45.4 ± 1.8 |     43.7 |     50.0 |       7.3 |       297.6 |        1.00 |
| `./target/release/rm_rayon /tmp/ftzz`          |  51.6 ± 1.2 |     50.0 |     53.8 |      18.6 |       310.4 | 1.14 ± 0.05 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 171.2 ± 1.5 |    168.9 |    173.8 |       5.5 |       162.0 | 3.77 ± 0.15 |
| `find /tmp/ftzz -delete`                       | 179.9 ± 0.9 |    178.7 |    181.4 |      12.2 |       163.8 | 3.96 ± 0.16 |
| `rm -r /tmp/ftzz`                              | 181.5 ± 1.3 |    180.1 |    184.6 |      12.4 |       165.4 | 4.00 ± 0.16 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 208.0 ± 1.5 |    206.0 |    209.9 |      17.3 |       186.1 | 4.58 ± 0.19 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 233.8 ± 0.9 |    232.5 |    235.1 |      14.4 |       176.7 | 5.15 ± 0.21 |

#### `remove_10_files_0_bytes.md`

| Command                                        |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:-----------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/rmz /tmp/ftzz`*             |  1.1 ± 0.1 |      1.0 |      1.4 |       0.6 |         1.6 |         1.00 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` |  1.3 ± 0.0 |      1.2 |      1.4 |       0.4 |         0.8 |  1.18 ± 0.07 |
| `./target/release/rm_rayon /tmp/ftzz`          |  1.5 ± 0.4 |      1.2 |      3.5 |       1.2 |         2.9 |  1.39 ± 0.34 |
| `./target/release/rm_stdlib /tmp/ftzz`         |  2.8 ± 0.1 |      2.5 |      3.2 |       0.7 |         1.9 |  2.53 ± 0.16 |
| `rm -r /tmp/ftzz`                              |  3.5 ± 0.1 |      3.3 |      3.7 |       1.0 |         2.3 |  3.25 ± 0.20 |
| `find /tmp/ftzz -delete`                       |  4.2 ± 0.3 |      3.5 |      4.8 |       1.0 |         2.6 |  3.85 ± 0.38 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 46.8 ± 0.1 |     46.5 |     47.0 |       2.4 |         5.1 | 42.89 ± 2.40 |

#### `remove_10_files_1G_bytes.md`

| Command                                        |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:-----------------------------------------------|-----------:|---------:|---------:|----------:|------------:|------------:|
| `./target/release/rm_rayon /tmp/ftzz`          | 15.9 ± 1.1 |     14.1 |     17.7 |       1.0 |        69.2 |        1.00 |
| *`./target/release/rmz /tmp/ftzz`*             | 17.3 ± 0.6 |     16.2 |     18.3 |       0.7 |        61.5 | 1.09 ± 0.08 |
| `./target/release/rm_stdlib /tmp/ftzz`         | 48.4 ± 0.3 |     47.4 |     49.1 |       0.3 |        47.4 | 3.04 ± 0.20 |
| `./target/release/rm_remove_dir_all /tmp/ftzz` | 48.5 ± 0.3 |     47.5 |     48.8 |       0.3 |        47.6 | 3.05 ± 0.20 |
| `rm -r /tmp/ftzz`                              | 49.0 ± 1.9 |     48.0 |     57.7 |       0.5 |        47.7 | 3.08 ± 0.24 |
| `find /tmp/ftzz -delete`                       | 51.2 ± 8.9 |     48.9 |     91.1 |       0.3 |        48.0 | 3.22 ± 0.60 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz`      | 90.3 ± 0.2 |     90.0 |     90.8 |       1.5 |        48.5 | 5.68 ± 0.38 |

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

| Command                                                                             |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  5.076 ± 0.976 |   4.283 |   6.700 |    2.769 |     23.713 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  5.854 ± 0.310 |   5.542 |   6.429 |    5.287 |     29.890 | 1.15 ± 0.23 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 11.801 ± 0.085 |  11.727 |  11.993 |    5.376 |     59.341 | 2.33 ± 0.45 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 11.829 ± 0.066 |  11.733 |  11.920 |    5.420 |     39.040 | 2.33 ± 0.45 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 15.019 ± 0.079 |  14.915 |  15.131 |    3.893 |     14.808 | 2.96 ± 0.57 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 18.436 ± 0.097 |  18.313 |  18.622 |    2.836 |     15.261 | 3.63 ± 0.70 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 19.496 ± 0.090 |  19.346 |  19.609 |    3.217 |     15.928 | 3.84 ± 0.74 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 28.195 ± 0.159 |  27.929 |  28.439 |    6.635 |     23.035 | 5.55 ± 1.07 |

#### `copy_1M_files_1G_bytes.md`

| Command                                                                             |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  7.593 ± 0.735 |   7.141 |   9.648 |    3.151 |     40.109 |         1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  8.827 ± 0.796 |   8.104 |  10.143 |    5.482 |     46.376 |  1.16 ± 0.15 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 13.374 ± 0.149 |  13.103 |  13.559 |    6.218 |     46.409 |  1.76 ± 0.17 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 13.459 ± 0.087 |  13.349 |  13.636 |    5.837 |     68.041 |  1.77 ± 0.17 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 23.669 ± 1.371 |  21.564 |  25.593 |    5.858 |     22.336 |  3.12 ± 0.35 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 28.173 ± 0.091 |  28.047 |  28.343 |    2.950 |     24.495 |  3.71 ± 0.36 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 28.945 ± 0.121 |  28.744 |  29.084 |    3.394 |     24.931 |  3.81 ± 0.37 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 89.173 ± 1.821 |  87.317 |  94.034 |   12.557 |     55.997 | 11.74 ± 1.16 |

#### `copy_100_000_files_0_bytes.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.379 ± 0.009 |   0.373 |   0.397 |    0.261 |      1.910 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.487 ± 0.018 |   0.471 |   0.535 |    0.513 |      2.604 | 1.28 ± 0.06 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.597 ± 0.016 |   0.581 |   0.629 |    0.554 |      3.073 | 1.57 ± 0.05 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 0.957 ± 0.015 |   0.939 |   0.982 |    0.529 |      3.295 | 2.52 ± 0.07 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 1.302 ± 0.022 |   1.277 |   1.332 |    0.389 |      1.274 | 3.43 ± 0.10 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.632 ± 0.023 |   1.611 |   1.682 |    0.277 |      1.323 | 4.30 ± 0.11 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 1.730 ± 0.009 |   1.719 |   1.750 |    0.308 |      1.389 | 4.56 ± 0.11 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 2.838 ± 0.019 |   2.803 |   2.864 |    0.638 |      2.141 | 7.49 ± 0.18 |

#### `copy_100_000_files_0_bytes_0_depth.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 1.133 ± 0.014 |   1.109 |   1.161 |    0.474 |      3.606 |        1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 1.260 ± 0.010 |   1.247 |   1.280 |    0.412 |      6.386 | 1.11 ± 0.02 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 1.270 ± 0.010 |   1.256 |   1.287 |    0.390 |      6.620 | 1.12 ± 0.02 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 1.365 ± 0.005 |   1.357 |   1.373 |    0.167 |      1.171 | 1.20 ± 0.02 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 1.406 ± 0.012 |   1.389 |   1.428 |    0.339 |      1.394 | 1.24 ± 0.02 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.618 ± 0.012 |   1.594 |   1.636 |    0.255 |      1.335 | 1.43 ± 0.02 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 1.735 ± 0.008 |   1.725 |   1.751 |    0.287 |      1.418 | 1.53 ± 0.02 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 2.157 ± 0.030 |   2.117 |   2.216 |    0.581 |      2.057 | 1.90 ± 0.04 |

#### `copy_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.413 ± 0.013 |   0.404 |   0.440 |    0.318 |      2.214 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.493 ± 0.008 |   0.484 |   0.509 |    0.576 |      2.661 | 1.19 ± 0.04 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.505 ± 0.009 |   0.496 |   0.523 |    0.672 |      2.700 | 1.22 ± 0.05 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 0.844 ± 0.033 |   0.804 |   0.897 |    0.723 |      2.841 | 2.04 ± 0.10 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 1.319 ± 0.018 |   1.297 |   1.350 |    0.463 |      1.371 | 3.19 ± 0.11 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 1.737 ± 0.008 |   1.729 |   1.757 |    0.318 |      1.385 | 4.20 ± 0.14 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 1.791 ± 0.012 |   1.767 |   1.805 |    0.336 |      1.419 | 4.33 ± 0.14 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 3.028 ± 0.023 |   2.991 |   3.064 |    0.787 |      2.217 | 7.33 ± 0.24 |

#### `copy_100_000_files_1G_bytes.md`

| Command                                                                             |       Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|---------------:|--------:|--------:|---------:|-----------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  0.711 ± 0.129 |   0.618 |   1.064 |    0.317 |      3.421 |         1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  0.819 ± 0.042 |   0.763 |   0.878 |    0.551 |      4.251 |  1.15 ± 0.22 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  0.858 ± 0.030 |   0.811 |   0.897 |    0.591 |      4.541 |  1.21 ± 0.22 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  1.110 ± 0.016 |   1.079 |   1.129 |    0.610 |      3.930 |  1.56 ± 0.28 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  2.437 ± 0.061 |   2.377 |   2.565 |    0.726 |      2.784 |  3.43 ± 0.63 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  2.453 ± 0.016 |   2.430 |   2.478 |    0.289 |      2.108 |  3.45 ± 0.63 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  2.550 ± 0.029 |   2.513 |   2.616 |    0.331 |      2.161 |  3.59 ± 0.65 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 10.653 ± 0.120 |  10.454 |  10.842 |    4.030 |      6.585 | 14.99 ± 2.73 |

#### `copy_100_000_files_1G_bytes_0_depth.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 1.209 ± 0.020 |   1.181 |   1.244 |    0.524 |      4.290 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 1.340 ± 0.005 |   1.334 |   1.350 |    0.402 |      6.999 | 1.11 ± 0.02 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 1.358 ± 0.015 |   1.338 |   1.391 |    0.432 |      6.779 | 1.12 ± 0.02 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 2.080 ± 0.012 |   2.064 |   2.099 |    0.206 |      1.820 | 1.72 ± 0.03 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 2.371 ± 0.020 |   2.348 |   2.408 |    0.288 |      2.030 | 1.96 ± 0.04 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 2.422 ± 0.026 |   2.386 |   2.477 |    0.638 |      2.749 | 2.00 ± 0.04 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 2.464 ± 0.037 |   2.432 |   2.541 |    0.268 |      2.133 | 2.04 ± 0.05 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 9.292 ± 0.080 |   9.164 |   9.382 |    3.508 |      5.456 | 7.69 ± 0.14 |

#### `copy_100_000_files_1G_bytes_0_depth_0_entropy.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 1.200 ± 0.018 |   1.178 |   1.230 |    0.539 |      4.253 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 1.346 ± 0.010 |   1.332 |   1.361 |    0.407 |      7.095 | 1.12 ± 0.02 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 1.409 ± 0.033 |   1.345 |   1.451 |    0.448 |      6.888 | 1.17 ± 0.03 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 2.086 ± 0.015 |   2.057 |   2.104 |    0.212 |      1.819 | 1.74 ± 0.03 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 2.331 ± 0.019 |   2.308 |   2.366 |    0.275 |      2.004 | 1.94 ± 0.03 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 2.364 ± 0.008 |   2.356 |   2.380 |    0.296 |      2.013 | 1.97 ± 0.03 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 2.406 ± 0.034 |   2.366 |   2.488 |    0.628 |      2.734 | 2.00 ± 0.04 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 9.245 ± 0.059 |   9.144 |   9.332 |    3.526 |      5.482 | 7.70 ± 0.13 |

#### `copy_100_000_files_1G_bytes_5_files_per_dir.md`

| Command                                                                             |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |     Relative |
|:------------------------------------------------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|-------------:|
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    | 0.733 ± 0.028 |   0.714 |   0.812 |    0.598 |      3.970 |         1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          | 0.766 ± 0.027 |   0.742 |   0.826 |    0.695 |      4.110 |  1.05 ± 0.05 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       | 0.775 ± 0.063 |   0.691 |   0.880 |    0.385 |      3.810 |  1.06 ± 0.10 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       | 1.050 ± 0.025 |   1.021 |   1.094 |    0.745 |      3.788 |  1.43 ± 0.07 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 2.359 ± 0.033 |   2.324 |   2.406 |    0.771 |      2.752 |  3.22 ± 0.13 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 2.506 ± 0.017 |   2.480 |   2.526 |    0.338 |      2.110 |  3.42 ± 0.13 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 2.540 ± 0.018 |   2.501 |   2.559 |    0.366 |      2.116 |  3.46 ± 0.14 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 9.930 ± 0.168 |   9.748 |  10.341 |    4.235 |      6.487 | 13.54 ± 0.57 |

#### `copy_10_000_files_0_bytes.md`

| Command                                                                             |   Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |    Relative |
|:------------------------------------------------------------------------------------|------------:|---------:|---------:|----------:|------------:|------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  33.6 ± 0.4 |     33.1 |     34.7 |      28.8 |       186.4 |        1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  43.7 ± 0.5 |     42.9 |     45.3 |      55.6 |       243.4 | 1.30 ± 0.02 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  45.7 ± 0.7 |     44.9 |     48.0 |      61.5 |       249.9 | 1.36 ± 0.03 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  78.2 ± 5.3 |     71.6 |     96.4 |      59.0 |       271.6 | 2.33 ± 0.16 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 119.5 ± 1.7 |    116.7 |    122.6 |      44.7 |       118.7 | 3.55 ± 0.07 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        | 159.8 ± 2.5 |    156.7 |    164.3 |      29.7 |       126.2 | 4.75 ± 0.09 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   | 164.0 ± 1.5 |    162.2 |    167.9 |      31.2 |       129.0 | 4.87 ± 0.07 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 321.1 ± 3.0 |    316.5 |    324.9 |      73.3 |       208.2 | 9.55 ± 0.15 |

#### `copy_10_000_files_1G_bytes.md`

| Command                                                                             |     Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------------------------------------------------|--------------:|---------:|---------:|----------:|------------:|-------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |    55.0 ± 2.9 |     52.0 |     59.7 |      34.0 |       313.3 |         1.00 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |    66.2 ± 3.7 |     62.9 |     72.7 |      55.6 |       374.0 |  1.20 ± 0.09 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |    67.9 ± 4.0 |     63.7 |     75.4 |      63.6 |       380.9 |  1.23 ± 0.10 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  142.2 ± 44.9 |     93.2 |    197.6 |      72.8 |       379.1 |  2.59 ± 0.83 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |   231.3 ± 2.5 |    228.8 |    235.2 |      30.9 |       194.4 |  4.21 ± 0.23 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |   234.7 ± 2.7 |    231.4 |    240.0 |      36.0 |       192.6 |  4.27 ± 0.23 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |   676.7 ± 4.2 |    670.8 |    683.0 |     185.0 |       915.5 | 12.31 ± 0.66 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 2572.6 ± 26.8 |   2535.0 |   2626.5 |    2714.9 |      1197.5 | 46.78 ± 2.55 |

#### `copy_10_files_0_bytes.md`

| Command                                                                             |  Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |     Relative |
|:------------------------------------------------------------------------------------|-----------:|---------:|---------:|----------:|------------:|-------------:|
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |  2.7 ± 0.3 |      2.3 |      3.7 |       2.3 |         5.2 |         1.00 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |  2.7 ± 0.1 |      2.6 |      3.3 |       0.7 |         1.8 |  1.01 ± 0.11 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |  2.8 ± 0.1 |      2.5 |      3.2 |       2.2 |         5.4 |  1.03 ± 0.13 |
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |  2.9 ± 0.2 |      2.6 |      3.5 |       1.4 |         4.8 |  1.08 ± 0.14 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |  3.4 ± 0.4 |      3.0 |      4.0 |       0.9 |         2.3 |  1.26 ± 0.20 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |  3.6 ± 0.4 |      3.2 |      4.6 |       1.7 |         3.2 |  1.33 ± 0.21 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` |  9.6 ± 1.4 |      6.7 |     10.9 |       6.6 |         7.8 |  3.56 ± 0.64 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 47.7 ± 0.1 |     47.2 |     48.0 |       3.2 |         5.7 | 17.77 ± 1.96 |

#### `copy_10_files_1G_bytes.md`

| Command                                                                             |    Mean [ms] | Min [ms] | Max [ms] | User [ms] | System [ms] |       Relative |
|:------------------------------------------------------------------------------------|-------------:|---------:|---------:|----------:|------------:|---------------:|
| *`./target/release/cpz /tmp/ftzz /tmp/ftzzz`*                                       |    1.3 ± 0.2 |      1.1 |      2.2 |       0.8 |         2.1 |           1.00 |
| `fcp /tmp/ftzz /tmp/ftzzz`                                                          |    1.5 ± 0.1 |      1.3 |      1.9 |       1.1 |         3.1 |    1.11 ± 0.15 |
| `./target/release/cp_rayon /tmp/ftzz /tmp/ftzzz`                                    |    1.9 ± 0.2 |      1.5 |      7.6 |       1.4 |         3.9 |    1.39 ± 0.23 |
| `cp -r /tmp/ftzz /tmp/ftzzz`                                                        |    1.9 ± 0.4 |      1.6 |      2.7 |       0.5 |         1.3 |    1.40 ± 0.31 |
| `./target/release/cp_stdlib /tmp/ftzz /tmp/ftzzz`                                   |    3.0 ± 0.1 |      2.8 |      3.2 |       0.7 |         2.1 |    2.23 ± 0.26 |
| `xcp -r /tmp/ftzz /tmp/ftzzz`                                                       |    3.4 ± 0.4 |      2.7 |     11.4 |       1.8 |         3.6 |    2.55 ± 0.40 |
| `sh -c '(cd /tmp/ftzz; tar cf - .) \| (mkdir /tmp/ftzzz; cd /tmp/ftzzz; tar xf -)'` | 304.3 ± 14.9 |    297.9 |    346.4 |      83.4 |       449.5 | 225.47 ± 27.89 |
| `rsync -rlp --inplace /tmp/ftzz /tmp/ftzzz`                                         | 1042.4 ± 1.7 |   1040.5 |   1045.6 |    1529.1 |       350.0 | 772.37 ± 87.80 |
