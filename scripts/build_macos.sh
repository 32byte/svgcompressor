PATH="$(pwd)/osxcross/target/bin:$PATH" &&
cargo build --release --target x86_64-apple-darwin &&
cp ./target/x86_64-apple-darwin/release/svgcomp ./bin/svgcomp_macos