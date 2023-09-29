# Token Counting and Visualization

## Installation and Usage

1. Install tools to compile Rust programs. Follow steps at https://rustup.rs/
2. In the root of this directory, run `cargo build --release` to build an executable compiled with optimizations at target/release/hw0
3. The program follows a command line interface. Prompting `--help` will give full options. Example usage: `./target/release/hw0 text.txt --lower --stem --stop -o output.txt`
4. The word counts will output either to the given file or to stdout. The visualization will be saved as `plot.html`
