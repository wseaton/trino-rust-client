# trino-codegen

Utility for automatically generating Rust structs from Trino queries by using a database connection to do codegen.

Required environment variables:

```sh
TRINO_USER=jsmith
TRINO_HOST=https://my.starburt.cluster.com
TRINO_PORT=443
```

To use, just install the binary and then run the command:

```sh
❯ trino-codegen --help
Usage: trino-codegen [OPTIONS]

Options:
  -i, --input-path <INPUT_PATH>    [default: ./src/queries/*.sql]
  -o, --output-path <OUTPUT_PATH>  [default: ./src/generated_structs.rs]
  -h, --help                       Print help
```

## Dependencies

Because `trino-codegen` pulls in some custom functions for serde, you may need to add these dependencies to your project's `Cargo.toml` to actually use your generated structs:

```toml
# for date types
chrono = "*"
# for the UUID trino type
uuid = { version = "*", features = ["v4", "serde"] }
# VARBINARY
data-encoding = "*"
# BIGDECIMAL
bigdecimal = { version = "*", features = ["serde"] }
```
