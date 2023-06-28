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
| `../pks/target/release/packs check` | 7.870 ± 0.379 | 7.468 | 8.222 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 70.172 ± 5.335 | 66.240 | 76.246 | 8.92 ± 0.80 |


## Hot Cache, without Spring
```
hyperfine --warmup=1 --runs=3 --export-markdown hot-cache-without-spring.md \
  '../pks/target/release/packs check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/packs check` | 3.801 ± 0.046 | 3.771 | 3.854 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 63.196 ± 18.210 | 42.169 | 73.746 | 16.63 ± 4.80 |
