#!/bin/bash

# Check if the file exists before removing it
if [ -f "tmp/packs_benchmarks.md" ]; then
  rm tmp/packs_benchmarks.md
fi

echo "I use https://github.com/sharkdp/hyperfine to benchmark, which makes it easy to get consistent benchmarks. Note that benchmarks are done with cache only. While it's interesting to see the performance improvement on a cold cache, it's not representative of the performance of the tool in a real-world scenario, since most of the time the cache will be warm." >> tmp/packs_benchmarks.md
echo "To run these benchmarks on your application, you can place this repo next to your rails application and run ../pks/dev/run_benchmarks.sh from the root of your application" >> tmp/packs_benchmarks.md

echo -e "\n## Hot Cache, with and without spring, entire codebase" >> tmp/packs_benchmarks.md

hyperfine --warmup=2 --runs=3 --export-markdown tmp/bm.md \
  '../pks/target/release/pks update' \
  '../pks/target/release/pks --experimental-parser update' \
  'DISABLE_SPRING=1 bin/packwerk update' \
  'bin/packwerk update'

cat tmp/bm.md >> tmp/packs_benchmarks.md

echo -e "\n## Hot Cache, with and without spring, single file" >> tmp/packs_benchmarks.md

hyperfine --warmup=2 --runs=3 --export-markdown tmp/bm.md \
  '../pks/target/release/pks check config/initializers/inflections.rb' \
  '../pks/target/release/pks --experimental-parser check config/initializers/inflections.rb' \
  'DISABLE_SPRING=1 bin/packwerk check config/initializers/inflections.rb' \
  'bin/packwerk check config/initializers/inflections.rb'

cat tmp/bm.md >> tmp/packs_benchmarks.md

mv tmp/packs_benchmarks.md ../pks/BENCHMARKS.md
