# ass6510

A small, self-contained Rust port of the assembler in [`old/`](old/README). It assembles 6510/6502 source to headerless object code or to a loadable module with a two-byte little-endian load address. It also offers a disassembler, hex dumps, an opcode list, and conversion between plain and loadable object files. The `old/` tree is left as historical source.

## Build

```sh
cargo build --release
cargo test
```

The executable is `target/release/ass6510`; there are no third-party crates.

## Assemble

```sh
ass6510 program.asm                   # writes program.asm.6510
ass6510 -m -o program.prg program.asm  # includes the load address
ass6510 -w -s symbols.txt program.asm
ass6510 --dry-run -v program.asm      # validates without writing
```

A source file begins with `ASS <address>` and normally ends with `END`. Mnemonics and directives must be indented when they have no label; a word in column one is treated as a label. `;` starts a line comment, and a line starting with `*` is also a comment.

```asm
        ASS $0801
start   LDA #$01
        STA $d020
        BNE .done
        BYT "Hi",0
.done   RTS
        END
```

Supported directives are `ASS`, `END`, `INCLUDE`, `BYT`, `ADR`, `DSB`, `DSA`, `EQU` (or `=`), `SET`, `ORG`, `ALG`, `BEC`, and `ENC`. `ORG` and `ALG` fill gaps with NOP (`$EA`); `DSB` and `DSA` fill with zero bytes. Local labels start with `.` and are scoped until the next global label. Forward references are patched when symbols are defined. A forward address uses an absolute instruction; a known address below `$100` can use zero-page encoding. Writing an address as four hex digits, such as `$0001`, forces absolute encoding.

`INCLUDE "file.asm"` inserts another source file at that line. Paths are relative to the file containing the directive, nested includes are supported up to 32 files, and labels and the program counter continue across file boundaries. Include files normally omit their own `ASS` and `END`; if present, those directives have the same effect as if written in the parent file. Recursive include cycles are errors.

```asm
        ASS $1000
        INCLUDE "sprites/characters.asm"
        END
```

Numbers can be decimal (`10`), hex (`$0A` or `0xaa`), octal (`&12` or `$$12`), binary (`%1010` or `0b1010`), base four (`=22`), or a character (`'A` or `'A'`). For character ROMs, `BYT 0b00111100,0b01000010,0b10000001` inserts three bytes. Character immediates include punctuation, for example `LDA #'"'`. Expressions support unary `+`, `-`, `~` and binary `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `<`, `>`. For compatibility with the old implementation, binary expressions are **right associative** and have no precedence: `10-3-2` evaluates to `9`.

Visual binary literals use `.` for zero and `#` for one, with exactly eight positions per byte:

```asm
        BYT %"..###...",%"#......#",%"########"
```

## Inspect and convert object code

```sh
ass6510 -d -m program.prg       # disassemble a loadable module
ass6510 -S -O 2048 program.bin  # emit reassemblable source
ass6510 -D program.bin          # 8-byte hex dump; -W gives 16 bytes
ass6510 --opcode-list
ass6510 -M 2048 program.bin     # convert in place to a module
ass6510 -P program.bin          # remove a module header in place
```

`-m` tells the inspection commands to read the first two bytes as the load address. `-O` supplies an address for plain object code. `--no-dsb`, `--no-nop`, and `--illegal` control disassembly display. `-M` and `-P` change the input file in place; use `--dry-run` to preview the operation. Use `--help` for all options.

The assembler reports errors with source line numbers and returns a nonzero exit code without writing an object file when assembly fails. Warnings are printed with `-w`.

## License

The Rust implementation is distributed under the Boost Software License, Version 1.0; see [LICENSE](LICENSE). The historical C++ implementation in `old/` retains its original license notices.
