I use https://github.com/sharkdp/hyperfine to benchmark, which makes it easy to get consistent benchmarks.

## Cold Cache, without Spring
```
hyperfine --runs=3 --export-markdown cold-cache-without-spring.md \
  --prepare 'rm -rf tmp/cache/packwerk' \
  '../pks/target/release/packs check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/packs check` | 7.413 ± 0.466 | 6.900 | 7.808 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 88.359 ± 6.480 | 84.431 | 95.838 | 11.92 ± 1.15 |

## Hot Cache, without Spring
```
hyperfine --warmup=1 --runs=3 --export-markdown hot-cache-without-spring.md \
  '../pks/target/release/packs check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/packs check` | 1.858 ± 0.036 | 1.829 | 1.899 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 36.807 ± 6.487 | 32.523 | 44.271 | 19.81 ± 3.51 |


```
Benchmark 1: ../pks/target/release/packs check
  Time (mean ± σ):      1.858 s ±  0.036 s    [User: 7.681 s, System: 4.603 s]
  Range (min … max):    1.829 s …  1.899 s    3 runs

Benchmark 2: DISABLE_SPRING=1 bin/packwerk check
  Time (mean ± σ):     36.807 s ±  6.487 s    [User: 43.691 s, System: 40.739 s]
  Range (min … max):   32.523 s … 44.271 s    3 runs

Summary
  ../pks/target/release/packs check ran
   19.81 ± 3.51 times faster than DISABLE_SPRING=1 bin/packwerk check
  ```
