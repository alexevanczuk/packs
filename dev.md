# TODO

# Implementation plan
## Create Steel Thread
- Get enough functionality working that `packs update` generates `package_todo.yml` files that can have a similarity score (+1 for violations in common, -1 for each difference), track progress against monolith.
- Identify easy ways to replicate packwerk behavior without porting over too many idiosyncracies.
- If there are ways to remove patterns in our codebase (e.g. `module Foo::Bar`) that might cause packs to diverge from packwerk, go for that
