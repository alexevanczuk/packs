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
| `../pks/target/release/packs check` | 3.429 ± 0.130 | 3.322 | 3.573 | 1.00 |
| `DISABLE_SPRING=1 bin/packwerk check` | 40.475 ± 3.635 | 38.224 | 44.668 | 11.80 ± 1.15 |
