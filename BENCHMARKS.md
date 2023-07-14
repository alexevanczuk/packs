I use https://github.com/sharkdp/hyperfine to benchmark, which makes it easy to get consistent benchmarks. Note that benchmarks are done with cache only. While it's interesting to see the performance improvement on a cold cache, it's not representative of the performance of the tool in a real-world scenario, since most of the time the cache will be warm.
To run these benchmarks on your application, you can place this repo next to your rails application and run bash ../pks/dev/run_benchmarks.sh from the root of your application

## Hot Cache, with and without spring, entire codebase
| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/pks update` | 2.219 ± 0.221 | 2.049 | 2.469 | 1.00 |
| `../pks/target/release/pks --experimental-parser update` | 2.506 ± 0.260 | 2.316 | 2.803 | 1.13 ± 0.16 |
| `DISABLE_SPRING=1 bin/packwerk update` | 29.653 ± 2.329 | 27.122 | 31.706 | 13.37 ± 1.70 |
| `bin/packwerk update` | 21.439 ± 2.535 | 19.080 | 24.120 | 9.66 ± 1.49 |

## Hot Cache, with and without spring, single file
| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `../pks/target/release/pks check config/initializers/inflections.rb` | 579.9 ± 22.7 | 564.6 | 606.0 | 1.00 |
| `../pks/target/release/pks --experimental-parser check config/initializers/inflections.rb` | 1041.3 ± 10.6 | 1031.7 | 1052.7 | 1.80 ± 0.07 |
| `DISABLE_SPRING=1 bin/packwerk check config/initializers/inflections.rb` | 16693.2 ± 455.8 | 16361.6 | 17213.0 | 28.79 ± 1.37 |
| `bin/packwerk check config/initializers/inflections.rb` | 6749.6 ± 106.0 | 6658.2 | 6865.8 | 11.64 ± 0.49 |
