I use https://github.com/sharkdp/hyperfine to benchmark, which makes it easy to get consistent benchmarks. Note that benchmarks are done with cache only. While it's interesting to see the performance improvement on a cold cache, it's not representative of the performance of the tool in a real-world scenario, since most of the time the cache will be warm.

## Hot Cache, with and without spring, entire codebase
| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/pks update` | 2.204 ± 0.092 | 2.103 | 2.284 | 1.00 |
| `../pks/target/release/pks --experimental-parser update` | 2.770 ± 0.064 | 2.700 | 2.825 | 1.26 ± 0.06 |
| `DISABLE_SPRING=1 bin/packwerk update` | 32.825 ± 3.670 | 30.010 | 36.976 | 14.90 ± 1.78 |
| `bin/packwerk update` | 19.229 ± 2.226 | 17.544 | 21.753 | 8.73 ± 1.07 |

## Hot Cache, with and without spring, single file
| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/pks check config/initializers/inflections.rb` | 585.5 ± 11.0 | 573.8 | 595.7 | 1.00 |
| `../pks/target/release/pks --experimental-parser check config/initializers/inflections.rb` | 1064.9 ± 22.4 | 1040.4 | 1084.3 | 1.82 ± 0.05 |
| `DISABLE_SPRING=1 bin/packwerk check config/initializers/inflections.rb` | 17143.4 ± 323.6 | 16790.5 | 17426.3 | 29.28 ± 0.78 |
| `bin/packwerk check config/initializers/inflections.rb` | 6549.9 ± 116.2 | 6439.4 | 6671.0 | 11.19 ± 0.29 |
