I use https://github.com/sharkdp/hyperfine to benchmark, which makes it easy to get consistent benchmarks.

## Cold Cache, without Spring
```
hyperfine --runs=3 --export-markdown cold-cache-without-spring.md \
  --prepare 'rm -rf tmp/cache/packwerk' \
  '../pks/target/release/pks check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/pks check` | 4.876 ± 0.415 | 4.448 | 5.277 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 65.161 ± 8.092 | 59.971 | 74.486 | 13.36 ± 2.01 |

```
Benchmark 1: ../pks/target/release/pks check
  Time (mean ± σ):      4.876 s ±  0.415 s    [User: 10.530 s, System: 6.823 s]
  Range (min … max):    4.448 s …  5.277 s    3 runs

Benchmark 2: DISABLE_SPRING=1 bin/packwerk check
  Time (mean ± σ):     65.161 s ±  8.092 s    [User: 207.615 s, System: 46.733 s]
  Range (min … max):   59.971 s … 74.486 s    3 runs

Summary
  ../pks/target/release/pks check ran
   13.36 ± 2.01 times faster than DISABLE_SPRING=1 bin/packwerk check
```

## Hot Cache, without Spring
```
hyperfine --warmup=1 --runs=3 --export-markdown hot-cache-without-spring.md \
  '../pks/target/release/pks check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/pks check` | 1.701 ± 0.016 | 1.691 | 1.719 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 30.510 ± 1.431 | 28.859 | 31.393 | 17.94 ± 0.86 |

```
Benchmark 1: ../pks/target/release/pks check
  Time (mean ± σ):      1.701 s ±  0.016 s    [User: 3.353 s, System: 4.176 s]
  Range (min … max):    1.691 s …  1.719 s    3 runs

Benchmark 2: DISABLE_SPRING=1 bin/packwerk check
  Time (mean ± σ):     30.510 s ±  1.431 s    [User: 35.742 s, System: 32.449 s]
  Range (min … max):   28.859 s … 31.393 s    3 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Summary
  ../pks/target/release/pks check ran
   17.94 ± 0.86 times faster than DISABLE_SPRING=1 bin/packwerk check
```
