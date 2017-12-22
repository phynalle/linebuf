linebuf
==========

The library provides a interface to read a line through a fixed size of buffer

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
linebuf = "0.0.1"
```

And add this to your crate root:
```rust
extern crate linebuf
```

## Example

```rust
extern crate linebuf;
use linebuf::{Line, LineReader};

let mut reader = LineReader::new(File::open("/path/to/file")?);
let mut buf = vec![0; 1024];
loop {
  match reader.try_read_line(&mut buf)? {
    Line::Return(0) => break, // EOF
    Line::Return(n) => {
      // reading data reached the `carriage return`(\n)
      ...
    }
    Line::More(n) => {
      // In this time, the data doesn't reached the end of line
      ...
    }
  }
}
```

