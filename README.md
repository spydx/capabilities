# Capabilities

Capability crate / lib for Rust

## Useful commands

To expand a test

```sh
> cargo expand --test svc
```

```sh
> cargo install cargo-expand
> cargo install cargo-watch
> cargo watch -q -c -x "expand --test svc"
```

Where svc is the test of code block you wanna run, and the result

```rust
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use capabilities::PoolSqlite;
use capabilities_derive::service;
use sqlx::Pool;
pub struct CapService {
    con: PoolSqlite,
}
pub struct CapServiceError;
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for CapServiceError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            CapServiceError => ::core::fmt::Formatter::write_str(f, "CapServiceError"),
        }
    }
}
impl CapService {
    pub async fn build(conf: String) -> Result<Self, crate::CapServiceError> {
        let con = Pool::connect(&conf)
            .await
            .expect("Failed to connect database");
        Ok(Self { con: con })
    }
}
#[allow(dead_code)]
fn main() -> Result<(), std::io::Error> {
    let body = async {
        let connection_string = "sqlite::memory:".to_string();
        let _pool = CapService::build(connection_string)
            .await
            .expect("Failed to create database");
        Ok(())
    };
    #[allow(clippy::expect_used)]
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(body)
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[])
}
```

## Testing with TryBuild

Every time we create a new test that has WIP output, we should verify that this is the output.
Then we can use, `cargo test` with the environment variable `TRYBUILD=overwrite` set to create `<testname>.stderr` for the tests to run properly.

```sh
> cargo test
    Finished test [unoptimized + debuginfo] target(s) in 0.12s
     Running unittests (target/debug/deps/capabilities-400d590ddccf7464)

... <other tests>

test service_db.rs [should pass] ... ok
test service_web.rs [should pass] ... ok
test database.rs [should fail to compile] ... ok
test service_struct.rs [should fail to compile] ... ok

... <other tests>
```

### New failing tests

For new failing tests we have to run the `cargo test` with the envariable `TRYBUILD`.

```sh
# set variable
> export TRYBUILD=overwrite
>
# then run cargo test */
>
# remove variable
> unset TRYBUILD
>
# output of all environment variables
> env 
```

## Sources

[Rust Macro tips](https://www.youtube.com/watch?v=5rwnWfMJflU)

[LogRocket Blog - Macros in Rust](https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/)
[LogRocket Blog - ProcMacro](https://blog.logrocket.com/procedural-macros-in-rust/)
