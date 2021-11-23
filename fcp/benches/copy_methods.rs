use std::{
    alloc,
    alloc::Layout,
    fs::{copy, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    os::unix::{fs::OpenOptionsExt, io::AsRawFd},
    path::PathBuf,
    time::Duration,
};

use cache_size::l1_cache_size;
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BatchSize, BenchmarkGroup, BenchmarkId,
    Criterion, Throughput,
};
use memmap2::{Mmap, MmapOptions};
use nix::{
    fcntl::{fallocate, FallocateFlags},
    libc::{off_t, O_DIRECT},
    sys::stat::{mknod, Mode, SFlag},
};
use rand::{thread_rng, RngCore};
use tempfile::{tempdir, TempDir};

// Don't use an OS backed tempfile since it might change the performance characteristics of our copy
struct NormalTempFile {
    dir: TempDir,
    from: PathBuf,
    to: PathBuf,
}

impl NormalTempFile {
    fn create(bytes: usize, direct_io: bool) -> NormalTempFile {
        if direct_io && bytes % (1 << 12) != 0 {
            panic!("Num bytes ({}) must be divisible by 2^12", bytes);
        }

        let dir = tempdir().unwrap();
        let from = dir.path().join("from");

        let mut buf = if direct_io {
            let layout = Layout::from_size_align(bytes, 1 << 12).unwrap();
            let ptr = unsafe { alloc::alloc(layout) };
            unsafe { Vec::<u8>::from_raw_parts(ptr, bytes, bytes) }
        } else {
            let mut v = Vec::with_capacity(bytes);
            // We write those bytes immediately after and dropping u8s does nothing
            #[allow(clippy::uninit_vec)]
            unsafe {
                v.set_len(bytes);
            }
            v
        };
        thread_rng().fill_bytes(buf.as_mut_slice());

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        if direct_io {
            options.custom_flags(O_DIRECT);
        }
        options.open(&from).unwrap().write_all(&buf).unwrap();

        NormalTempFile {
            to: dir.path().join("to"),
            dir,
            from,
        }
    }
}

/// Doesn't use direct I/O, so files will be mem cached
fn with_memcache(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_memcache");

    for num_bytes in [1 << 10, 1 << 20, 1 << 25] {
        add_benches(&mut group, num_bytes, false);
    }
}

/// Use direct I/O to create the file to be copied so it's not cached initially
fn initially_uncached(c: &mut Criterion) {
    let mut group = c.benchmark_group("initially_uncached");

    for num_bytes in [1 << 20] {
        add_benches(&mut group, num_bytes, true);
    }
}

fn empty_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("empty_files");

    group.throughput(Throughput::Elements(1));

    group.bench_function("copy_regular_files", |b| {
        b.iter_batched(
            || NormalTempFile::create(0, false),
            |files| {
                // Uses the copy_regular_files syscall on Linux
                copy(files.from, files.to).unwrap();
                files.dir
            },
            BatchSize::LargeInput,
        )
    });

    group.bench_function("open", |b| {
        b.iter_batched(
            || NormalTempFile::create(0, false),
            |files| {
                File::create(files.to).unwrap();

                files.dir
            },
            BatchSize::LargeInput,
        )
    });

    group.bench_function("mknod", |b| {
        b.iter_batched(
            || NormalTempFile::create(0, false),
            |files| {
                mknod(files.to.as_path(), SFlag::S_IFREG, Mode::empty(), 0).unwrap();

                files.dir
            },
            BatchSize::LargeInput,
        )
    });
}

fn add_benches(group: &mut BenchmarkGroup<WallTime>, num_bytes: u64, direct_io: bool) {
    group.throughput(Throughput::Bytes(num_bytes));

    group.bench_with_input(
        BenchmarkId::new("copy_regular_files", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    // Uses the copy_regular_files syscall on Linux
                    copy(files.from, files.to).unwrap();
                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let reader = BufReader::new(File::open(files.from).unwrap());
                    write_from_buffer(files.to, reader);
                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered_l1_tuned", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let l1_cache_size = l1_cache_size().unwrap();
                    let reader =
                        BufReader::with_capacity(l1_cache_size, File::open(files.from).unwrap());

                    write_from_buffer(files.to, reader);

                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered_readahead_tuned", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let readahead_size = 1 << 17; // See https://eklitzke.org/efficient-file-copying-on-linux
                    let reader =
                        BufReader::with_capacity(readahead_size, File::open(files.from).unwrap());

                    write_from_buffer(files.to, reader);

                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_read_only", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut to = File::create(files.to).unwrap();
                    advise(&from);

                    to.write_all(reader.as_ref()).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_read_only_truncate", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut to = File::create(files.to).unwrap();
                    advise(&from);
                    to.set_len(*num_bytes).unwrap();

                    to.write_all(reader.as_ref()).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_read_only_fallocate", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut to = File::create(files.to).unwrap();
                    advise(&from);
                    allocate(&to, *num_bytes);

                    to.write_all(reader.as_ref()).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_rw_truncate", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes as usize, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let to = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(files.to)
                        .unwrap();
                    to.set_len(*num_bytes).unwrap();
                    advise(&from);
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut writer = unsafe { MmapOptions::new().map_mut(&to) }.unwrap();

                    writer.copy_from_slice(reader.as_ref());

                    files.dir
                },
                BatchSize::PerIteration,
            )
        },
    );
}

fn write_from_buffer(to: PathBuf, mut reader: BufReader<File>) {
    advise(reader.get_ref());
    let mut to = File::create(to).unwrap();
    to.set_len(reader.get_ref().metadata().unwrap().len())
        .unwrap();

    loop {
        let len = {
            let buf = reader.fill_buf().unwrap();
            if buf.is_empty() {
                break;
            }

            to.write_all(buf).unwrap();
            buf.len()
        };
        reader.consume(len)
    }
}

fn allocate(file: &File, len: u64) {
    fallocate(file.as_raw_fd(), FallocateFlags::empty(), 0, len as off_t).unwrap();
}

fn advise(_file: &File) {
    // Interestingly enough, this either had no effect on performance or made it slightly worse.
    // posix_fadvise(file.as_raw_fd(), 0, 0, POSIX_FADV_SEQUENTIAL).unwrap();
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.02).warm_up_time(Duration::from_secs(1));
    targets =
    with_memcache,
    initially_uncached,
    empty_files,
}
criterion_main!(benches);
