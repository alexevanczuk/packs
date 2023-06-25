## Cold Cache, without Spring
- `packs check`: `rm -rf tmp/cache/packwerk && DISABLE_SPRING=1 time ../pks/target/release/packs check`
- `packwerk check`: `rm -rf tmp/cache/packwerk && DISABLE_SPRING=1 time bin/packwerk check`

| Run         | `packs check` | `packwerk check` |
|-------------|---------------|------------------|
| 1           | 8.9s          | 107.83s          |
| 2           | 7.31s         | 85.24s           |
| 3           | 7.55s         | 126.52s          |
| 4           | 6.85s         | 80.47s           |
| 5           | 8.45s         | 99.90s           |
| **Average** | 7.812s        | 99.99s           |

## Hot Cache, without Spring
- `packs check`: `DISABLE_SPRING=1 time ../pks/target/release/packs check`
- `packwerk check`: `DISABLE_SPRING=1 time bin/packwerk check`

| Run         | `packs check` | `packwerk check` |
|-------------|---------------|------------------|
| 1           | 3.86s         | 39.33s           |
| 2           | 3.69s         | 34.02s           |
| 3           | 3.6s          | 41.68s           |
| 4           | 3.52s         | 35.26s           |
| 5           | 3.32s         | 37.14s           |
| **Average** | 3.598         | 37.29            |
