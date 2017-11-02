##

Swift Package Manager had some trouble with system modules, so we rely on `swiftc` for now.

### Building

 - Run, `cargo build --release` to build the static Rust library.
 - Then, run `swiftc NaamioClient.swift -I ./Merileva/ -Xlinker -L../target/release/` to create the binary and link it to the library.
