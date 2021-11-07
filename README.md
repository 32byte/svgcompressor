# Svg Compressor

## Usage
```
Merges neighboring rectangles with the same attributes into one

USAGE:
    
     [FLAGS] [OPTIONS] <INPUT> <OUTPUT>

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -i, --iter <iterations>    Amount of iterations to compress [default: 8]

ARGS:
    <INPUT>     Input file to compress
    <OUTPUT>    Output file to save to
```


## Use-cases
If you have a svg with many rectangles with the same property next to each other, this tool will merge them into as little as possible rectangles without changing how the svg looks.


## How it works
Here is a simplified pseudo-code that shows how the program works:
```rust
for rect in rectangles {
    for neighbor in neighbors[rect] {
        if adjacent(rect, neighbor) 
            && same(rect, neighbor) {
            
            merge(rect, neighbor);
        }
    }
}
```

### Runtime complexity:
Worst case: O(N*N)
Best case: O(N)


## Known issues / shortcomings
 - When merging rectangles its not accounting for namespaces
 - Rect equality checking might not work in a specific use case
 - Merging works only on rectangles as of now
 - The library needs unit tests to be 100% sure everything works fine


## Building
To build the project from source you need to have a working rust installation.
Then run `cargo build --release`. The binary can be found in the target folder.
If you don't want to build the project use the prebuilt binaries.


## Cross-compiling

### Setup
```bash
# Linux
rustup target add x86_64-unknown-linux-gnu

# Windows
rustup target add x86_64-pc-windows-gnu

# MacOS
rustup target add x86_64-apple-darwin
# MacOS need a custom linker
bash ./scripts/osxcross_setup.sh
```

### Building
```bash
# Linux
bash ./scripts/build_linux.sh

# Windows
bash ./scripts/build_windows.sh

# MacOS
bash ./scripts/build_macos.sh
```

### Warning
The macos build is not tested since I don't have a mac. 
The prebuilt binary might not start and you would have to build the project from source!