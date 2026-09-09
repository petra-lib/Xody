# Xodyr 🤓

**Xody** is an interpreted programming language written in Rust, built as a learning project by following the [*Crafting Interpreters*](https://craftinginterpreters.com) book by Robert Nystrom.

## Project structure

The repository is organized as a Cargo workspace:

```
Xody/
├── Cargo.toml          # workspace root
├── crates/
│   └── xodyr/           # main crate: the Xody interpreter/runtime
├── test.xody             # sample program written in Xody
└── README.md
```

## Syntax

The `test.xody` file included in the repo gives an idea of the syntax, C/Lox-style:

```c
for(var i = 0; i < 10; i = i + 1){
  var ci = "test " + i;
  println ci;
}
```

TODO: Write all the syntax rules available

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain) and Cargo

## Build

Clone the repository and build it with Cargo:

```bash
git clone https://github.com/petra-lib/Xody.git
cd Xody
cargo build --release
```

## Usage

To run a `.xody` file via the `xodyr` crate:

```bash
cargo run -- test.xody
```
