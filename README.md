# fun_time

[![Crates.io](https://img.shields.io/crates/v/fun_time)](https://crates.io/crates/fun_time)
[![docs.rs](https://img.shields.io/docsrs/fun_time)](https://docs.rs/fun_time/0.2.0/fun_time/)

fun_time is a simple Rust library that allows you to easily time your function calls with a simple attribute!

### Basic example

```rust
#[fun_time]
fn some_cool_function(a_value: String) -> usize {
    a_value.len()
}

fn main() {
    let my_value_length = some_cool_function(String::from("Hello, world."));
}
```

### Configuration

There are various attributes that allow you to configure the behavior of the `fun_time` attribute.

- `message` allows you to set a message that will be printed when starting, and when done.
- `when` allows you to configure when the timing should be collected. The possible values for this are: `"always"` which
  as the name might suggest will always collect timing information, and `"debug"` which will only collect when
  `cfg!(debug_assertions)` evaluates to `true`.
- `give_back` is a flag that makes it so the wrapped function will now return the elapsed time instead of printing it.
  For this it modifies the return type from for example: `-> &'a str` to `-> (&'a str, std::time::Duration)`. This
  allows you to handle printing or storing the timing information.
- `reporting` (_can not be used in combination with give_back_) determines how the reporting is done. The possible
  options are: `"println"` which will print to stdout using `println!`. The `"log"` option is only available when
  the `log` feature is used. This will use the [log](https://crates.io/crates/log) crate with `info!` level logs.

#### Reporting

The reported messages are formatted as follows:

**Start message**: "Starting: YOUR_MESSAGE_HERE"

**Done message**: "YOUR_MESSAGE_HERE: Done in DURATION"
