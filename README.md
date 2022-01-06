# Compiling

`cargo build`

# Usage

### Generating a key:

Output via stdout.

`./target/debug/rust_x_pad -g <key_size_in_mb>`

### Encrypt/decrypt a message:

Input is via stdin, output is stdout

`./target/debug/rust_x_pad keyfile`
