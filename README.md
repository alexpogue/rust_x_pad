# Compile

`cargo build --release`

# Usage

### Generate a key:

Generate a 1mb key
```./rust_x_pad -g 1```
Generate a 100mb key and output to a file
```./rust_x_pad -g 100 -o secret.key```

### Encrypt/decrypt a message:

Input is via stdin, output is stdout. argument is a secret key file.
```./rust_x_pad secret.key````

To encrypt a file
``` "i love you" > ./rust_x_pad > cypher.data

