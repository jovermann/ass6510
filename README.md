# ass6510

A small, self-contained 6510/6502 assembler written in Rust. This is a port of the C++ version of ass6510, originally written 1993 under Borland C++ 6.0 under DOS and ported to Linux in 1994.
It assembles source to headerless object code or to a loadable module with a two-byte little-endian load address (PRG format).

Features:

- local labels
- nested includes (INCLUDE)
- string literals, character literals, binary and visual binary literals
- symbol table
- PRG format (address in first two bytes)
- disassembler
- hex dump
- opcode list
- object file conversion

## Build

```sh
cargo build --release
cargo test
```

The executable is `target/release/ass6510`

## Assemble

```sh
ass6510 program.asm                   # writes program.asm.6510
ass6510 -m -o program.prg program.asm # includes the load address
ass6510 -w -s symbols.txt program.asm # assembles `program.asm` into `program.asm.6510`, prints assembler warnings, and writes a listing of global symbols to `symbols.txt`.
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

## Directives

| Directive | Syntax | Description |
| --- | --- | --- |
| `ASS` | `ASS address` | Start assembly at `address`; required once at the beginning. |
| `END` | `END` | Stop assembly. |
| `INCLUDE` | `INCLUDE "file.asm"` | Insert another source file at this line. |
| `BYT` | `BYT value[,value...]` | Emit bytes; quoted strings emit their UTF-8 bytes. |
| `ADR` | `ADR address[,address...]` | Emit 16-bit addresses in little-endian order. |
| `DSB` | `DSB count` | Reserve `count` zero bytes. |
| `DSA` | `DSA count` | Reserve `count` zero-filled 16-bit words. |
| `EQU`, `=` | `name EQU value` or `name = value` | Define a symbol with a fixed value. |
| `SET` | `name SET value` | Define or redefine a symbol. |
| `ORG` | `ORG address` | Set the program counter; fill a forward gap with NOP (`$EA`). |
| `ALG` | `ALG power` | Align to a `2^power` address boundary, filling with NOP (`$EA`). |
| `BEC` | `BEC` | Begin a block comment; blocks can be nested. |
| `ENC` | `ENC` | End a block comment. |

Local labels start with `.` and are scoped until the next global label. Forward references are patched when symbols are defined. A forward address uses an absolute instruction; a known address below `$100` can use zero-page encoding. Writing an address as four hex digits, such as `$0001`, forces absolute encoding.

`INCLUDE "file.asm"` inserts another source file at that line. Paths are relative to the file containing the directive, nested includes are supported up to 32 files, and labels and the program counter continue across file boundaries. Include files normally omit their own `ASS` and `END`; if present, those directives have the same effect as if written in the parent file. Recursive include cycles are errors.

```asm
        ASS $1000
        INCLUDE "sprites/characters.asm"
        END
```

Numbers can be decimal (`10`), hex (`$0A` or `0xaa`), octal (`&12` or `$$12`), binary (`%1010` or `0b1010`), base four (`=22`), or a character (`'A` or `'A'`). For character ROMs, `BYT 0b00111100,0b01000010,0b10000001` inserts three bytes. Character immediates include punctuation, for example `LDA #'"'`. Expressions support unary `+`, `-`, `~` and binary `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `<`, `>`. Binary expressions are **right associative** and have no precedence: `10-3-2` evaluates to `9`.

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

`-M` and `-P` change the input file in place; use `--dry-run` to preview the operation.

## Command-line options

Assembler options:

- `-m`, `--module`: Prefix assembled output with its two-byte load address; read that address when inspecting a PRG file.
- `-o FILE`, `--output-file FILE`: Write assembled object code to `FILE` (default: `<infile>.6510`).
- `-s FILE`, `--symbol-file FILE`: Write a global symbol listing to `FILE`.
- `-w`, `--warnings`: Print assembler warnings.
- `-f`, `--stop-on-errors`: Show at most 20 errors.

Object-code options:

- `-d`, `--disassemble`: Print a disassembly.
- `-S`, `--disass-source`: Print reassemblable source.
- `-D`, `--hexdump`: Print an 8-byte-wide hex dump.
- `-W`, `--wide-hexdump`: Print a 16-byte-wide hex dump.
- `--opcode-list`: Print all 256 opcode slots.
- `--no-dsb`: Do not compress repeated zero bytes in disassembly.
- `--no-nop`: Do not compress repeated NOP bytes in disassembly.
- `--illegal`: Show unsupported opcodes as `???` in disassembly.
- `-O ADR`, `--start-address ADR`: Set the base address for headerless input.
- `-M ADR`, `--make-module ADR`: Add or replace a two-byte load address in the input file.
- `-P`, `--make-plain`: Remove the two-byte load address from the input file.

Common options:

- `-v`, `--verbose`: Show individual file operations.
- `--dry-run`: Validate and preview operations without writing files.
- `-h`, `--help`: Show help.
- `--version`: Show the version.
- `--`: Treat the following argument as a filename, even if it begins with `-`.

The assembler reports errors with source line numbers and returns a nonzero exit code without writing an object file when assembly fails. Warnings are printed with `-w`.

## License

This project is distributed under the Boost Software License, Version 1.0; see [LICENSE](LICENSE).
