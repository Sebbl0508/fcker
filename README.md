# fcker
A brainfuck interpreter.

## Specifics
- Memory Size
    - 30.000 Bytes
- Over-/Underflow integer
    - Wrapping (0->255 & 255->0)
- Over-/Underflow data pointer
    - Panic/Crash

## Compile
To compile you need to have the rust toolchain installed.
Then:
```
cargo build
# or
cargo build --release
```

And find your binary at `target/debug/fcker` or `target/release/fcker`.

## Run
Pass the path to the brainfuck code as the first argument:
```
./fcker hello_world.bf`
```