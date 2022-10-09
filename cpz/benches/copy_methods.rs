use std::{
    alloc,
    alloc::Layout,
    fs::{copy, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::unix::{fs::FileExt, io::AsRawFd},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use cache_size::l1_cache_size;
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, AxisScale, BatchSize, BenchmarkGroup,
    BenchmarkId, Criterion, PlotConfiguration, Throughput,
};
use memmap2::{Mmap, MmapOptions};
use rand::{thread_rng, RngCore};
use tempfile::{tempdir, TempDir};

// Don't use an OS backed tempfile since it might change the performance
// characteristics of our copy
struct NormalTempFile {
    dir: TempDir,
    from: PathBuf,
    to: PathBuf,
}

impl NormalTempFile {
    fn create(bytes: u64, direct_io: bool) -> Self {
        assert!(
            !direct_io || bytes % (1 << 12) == 0,
            "Num bytes ({}) must be divisible by 2^12",
            bytes
        );

        let dir = tempdir().unwrap();
        let from = dir.path().join("from");

        let buf = create_random_buffer(usize::try_from(bytes).unwrap(), direct_io);

        open_standard(&from, direct_io).write_all(&buf).unwrap();

        Self {
            to: dir.path().join("to"),
            dir,
            from,
        }
    }
}

/// Doesn't use direct I/O, so files will be mem cached
fn with_memcache(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_memcache");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_bytes in [1 << 10, 1 << 20, 1 << 25] {
        add_benches(&mut group, num_bytes, false);
    }
}

/// Use direct I/O to create the file to be copied so it's not cached initially
fn initially_uncached(c: &mut Criterion) {
    let mut group = c.benchmark_group("initially_uncached");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_bytes in [1 << 10, 1 << 20, 1 << 25] {
        add_benches(&mut group, num_bytes, true);
    }
}

fn empty_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("empty_files");

    group.throughput(Throughput::Elements(1));

    group.bench_function("fs::copy", |b| {
        b.iter_batched(
            || NormalTempFile::create(0, false),
            |files| {
                copy(files.from, files.to).unwrap();
                files.dir
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("open", |b| {
        b.iter_batched(
            || NormalTempFile::create(0, false),
            |files| {
                File::create(files.to).unwrap();

                files.dir
            },
            BatchSize::LargeInput,
        );
    });

    #[cfg(target_os = "linux")]
    group.bench_function("mknod", |b| {
        b.iter_batched(
            || NormalTempFile::create(0, false),
            |files| {
                use nix::sys::stat::{mknod, Mode, SFlag};
                mknod(files.to.as_path(), SFlag::S_IFREG, Mode::empty(), 0).unwrap();

                files.dir
            },
            BatchSize::LargeInput,
        );
    });
}

fn just_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("just_writes");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_bytes in [1 << 10, 1 << 20, 1 << 25] {
        group.throughput(Throughput::Bytes(num_bytes));

        group.bench_with_input(
            BenchmarkId::new("open_memcache", num_bytes),
            &num_bytes,
            |b, num_bytes| {
                b.iter_batched(
                    || {
                        let dir = tempdir().unwrap();
                        let buf = create_random_buffer(usize::try_from(*num_bytes).unwrap(), false);

                        (dir, buf)
                    },
                    |(dir, buf)| {
                        File::create(dir.path().join("file"))
                            .unwrap()
                            .write_all(&buf)
                            .unwrap();

                        (dir, buf)
                    },
                    BatchSize::PerIteration,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("open_nocache", num_bytes),
            &num_bytes,
            |b, num_bytes| {
                b.iter_batched(
                    || {
                        let dir = tempdir().unwrap();
                        let buf = create_random_buffer(usize::try_from(*num_bytes).unwrap(), true);

                        (dir, buf)
                    },
                    |(dir, buf)| {
                        let mut out = open_standard(dir.path().join("file").as_ref(), true);
                        out.set_len(*num_bytes).unwrap();

                        out.write_all(&buf).unwrap();

                        (dir, buf)
                    },
                    BatchSize::PerIteration,
                );
            },
        );
    }
}

#[allow(clippy::too_many_lines)]
fn add_benches(group: &mut BenchmarkGroup<WallTime>, num_bytes: u64, direct_io: bool) {
    group.throughput(Throughput::Bytes(num_bytes));

    group.bench_with_input(
        BenchmarkId::new("fs::copy", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    copy(files.from, files.to).unwrap();
                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    #[cfg(target_os = "linux")]
    group.bench_with_input(
        BenchmarkId::new("copy_file_range", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            use nix::fcntl::copy_file_range;

            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let to = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(files.to)
                        .unwrap();
                    to.set_len(*num_bytes).unwrap();

                    let mut bytes_remaining = usize::try_from(*num_bytes).unwrap();
                    while bytes_remaining > 0 {
                        bytes_remaining -= copy_file_range(
                            from.as_raw_fd(),
                            None,
                            to.as_raw_fd(),
                            None,
                            bytes_remaining,
                        )
                        .unwrap();
                    }

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let reader = BufReader::new(File::open(files.from).unwrap());
                    write_from_buffer(files.to, reader);
                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered_l1_tuned", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let l1_cache_size = l1_cache_size().unwrap();
                    let reader =
                        BufReader::with_capacity(l1_cache_size, File::open(files.from).unwrap());

                    write_from_buffer(files.to, reader);

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered_readahead_tuned", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let readahead_size = 1 << 17; // See https://eklitzke.org/efficient-file-copying-on-linux
                    let reader =
                        BufReader::with_capacity(readahead_size, File::open(files.from).unwrap());

                    write_from_buffer(files.to, reader);

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered_parallel", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let threads =
                        u64::try_from(thread::available_parallelism().unwrap().get()).unwrap();
                    let chunk_size = num_bytes / threads;

                    let from = File::open(files.from).unwrap();
                    let to = File::create(files.to).unwrap();
                    to.set_len(*num_bytes).unwrap();

                    let mut results = Vec::with_capacity(usize::try_from(threads).unwrap());
                    for i in 0..threads {
                        let from = from.try_clone().unwrap();
                        let to = to.try_clone().unwrap();

                        results.push(thread::spawn(move || {
                            let mut buf = Vec::with_capacity(usize::try_from(chunk_size).unwrap());
                            // We write those bytes immediately after and dropping u8s does nothing
                            #[allow(clippy::uninit_vec)]
                            unsafe {
                                buf.set_len(usize::try_from(chunk_size).unwrap());
                            }

                            from.read_exact_at(&mut buf, i * chunk_size).unwrap();
                            to.write_all_at(&buf, i * chunk_size).unwrap();
                        }));
                    }
                    for handle in results {
                        handle.join().unwrap();
                    }

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("buffered_entire_file", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let mut from = File::open(files.from).unwrap();
                    let mut to = File::create(files.to).unwrap();
                    to.set_len(*num_bytes).unwrap();

                    let mut buf = Vec::with_capacity(usize::try_from(*num_bytes).unwrap());
                    from.read_to_end(&mut buf).unwrap();
                    to.write_all(&buf).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_read_only", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut to = File::create(files.to).unwrap();

                    to.write_all(reader.as_ref()).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_read_only_truncate", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut to = File::create(files.to).unwrap();
                    to.set_len(*num_bytes).unwrap();

                    to.write_all(reader.as_ref()).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    #[cfg(target_os = "linux")]
    group.bench_with_input(
        BenchmarkId::new("mmap_read_only_fallocate", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut to = File::create(files.to).unwrap();
                    allocate(&to, *num_bytes);

                    to.write_all(reader.as_ref()).unwrap();

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("mmap_rw_truncate", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let to = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(files.to)
                        .unwrap();
                    to.set_len(*num_bytes).unwrap();
                    let reader = unsafe { Mmap::map(&from) }.unwrap();
                    let mut writer = unsafe { MmapOptions::new().map_mut(&to) }.unwrap();

                    writer.copy_from_slice(reader.as_ref());

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );

    #[cfg(target_os = "linux")]
    group.bench_with_input(
        BenchmarkId::new("sendfile", num_bytes),
        &num_bytes,
        |b, num_bytes| {
            use nix::sys::sendfile::sendfile64;

            b.iter_batched(
                || NormalTempFile::create(*num_bytes, direct_io),
                |files| {
                    let from = File::open(files.from).unwrap();
                    let to = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(files.to)
                        .unwrap();
                    to.set_len(*num_bytes).unwrap();

                    let mut bytes_remaining = usize::try_from(*num_bytes).unwrap();
                    while bytes_remaining > 0 {
                        bytes_remaining -=
                            sendfile64(to.as_raw_fd(), from.as_raw_fd(), None, bytes_remaining)
                                .unwrap();
                    }

                    files.dir
                },
                BatchSize::PerIteration,
            );
        },
    );
}

fn open_standard(path: &Path, direct_io: bool) -> File {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(target_os = "linux")]
    if direct_io {
        use nix::libc::O_DIRECT;
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(O_DIRECT);
    }

    let file = options.open(path).unwrap();

    #[cfg(target_os = "macos")]
    if direct_io {
        use nix::{
            errno::Errno,
            libc::{fcntl, F_NOCACHE},
        };
        Errno::result(unsafe { fcntl(file.as_raw_fd(), F_NOCACHE) }).unwrap();
    }

    #[allow(clippy::let_and_return)]
    file
}

fn write_from_buffer(to: PathBuf, mut reader: BufReader<File>) {
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
        reader.consume(len);
    }
}

#[cfg(target_os = "linux")]
fn allocate(file: &File, len: u64) {
    use nix::{
        fcntl::{fallocate, FallocateFlags},
        libc::off_t,
    };
    fallocate(
        file.as_raw_fd(),
        FallocateFlags::empty(),
        0,
        off_t::try_from(len).unwrap(),
    )
    .unwrap();
}

fn create_random_buffer(bytes: usize, direct_io: bool) -> Vec<u8> {
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
    buf
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.02).warm_up_time(Duration::from_secs(1));
    targets =
    with_memcache,
    initially_uncached,
    empty_files,
    just_writes,
}
criterion_main!(benches);
