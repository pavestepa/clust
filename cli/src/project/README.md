# Cargo application builds

The parent project module reads Clust.toml and resolves dependency paths.
The build module creates a Cargo package under target/clust, passes the
selected profile and dependencies to Cargo, and returns the executable path
and checked source. The modules helper stages the source directory layout for
mixed applications so rustc can resolve nested modules and relative assets.

Async applications add Tokio. [Project entry](../project.rs).
