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
      "./target/release/rmz /tmp/ftzz"
done
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

#### `remove_100_000_files_0_bytes_5_files_per_dir.md`

| Command                                   |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        | 0.820 ± 0.202 |   0.584 |   1.085 |    0.171 |      2.944 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     | 1.108 ± 0.218 |   0.799 |   1.473 |    0.404 |      3.617 | 1.35 ± 0.43 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 1.230 ± 0.055 |   1.178 |   1.352 |    0.107 |      1.046 | 1.50 ± 0.38 |
| `find /tmp/ftzz -delete`                  | 1.361 ± 0.026 |   1.328 |   1.416 |    0.185 |      1.110 | 1.66 ± 0.41 |
| `rm -r /tmp/ftzz`                         | 1.413 ± 0.030 |   1.380 |   1.479 |    0.186 |      1.147 | 1.72 ± 0.43 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz` | 1.497 ± 0.030 |   1.470 |   1.569 |    0.198 |      1.215 | 1.83 ± 0.45 |

#### `remove_100_000_files_100M_bytes_5_files_per_dir.md`

| Command                                   |      Mean [s] | Min [s] | Max [s] | User [s] | System [s] |    Relative |
|:------------------------------------------|--------------:|--------:|--------:|---------:|-----------:|------------:|
| *`./target/release/rmz /tmp/ftzz`*        | 0.666 ± 0.071 |   0.601 |   0.795 |    0.149 |      3.046 |        1.00 |
| `./target/release/rayon_rm /tmp/ftzz`     | 0.856 ± 0.100 |   0.723 |   1.000 |    0.311 |      3.500 | 1.29 ± 0.20 |
| `./target/release/stdlib_rm /tmp/ftzz`    | 1.454 ± 0.009 |   1.440 |   1.466 |    0.107 |      1.298 | 2.18 ± 0.23 |
| `find /tmp/ftzz -delete`                  | 1.609 ± 0.016 |   1.590 |   1.646 |    0.186 |      1.372 | 2.42 ± 0.26 |
| `rm -r /tmp/ftzz`                         | 1.640 ± 0.014 |   1.622 |   1.670 |    0.187 |      1.405 | 2.46 ± 0.26 |
| `rsync --delete -r /tmp/empty/ /tmp/ftzz` | 1.772 ± 0.026 |   1.737 |   1.836 |    0.198 |      1.498 | 2.66 ± 0.29 |

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
