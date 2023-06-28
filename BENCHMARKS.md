I use https://github.com/sharkdp/hyperfine to benchmark, which makes it easy to get consistent benchmarks.

## Cold Cache, without Spring
```
hyperfine --export-markdown cold-cache-without-spring.md \
  --prepare 'rm -rf tmp/cache/packwerk' \
  '../pks/target/release/packs check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```
## Hot Cache, without Spring
```
hyperfine --warmup=1 --export-markdown hot-cache-without-spring.md \
  '../pks/target/release/packs check' \
  'DISABLE_SPRING=1 bin/packwerk check'
```
