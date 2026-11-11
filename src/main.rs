// Copyright (c) 2026 Johannes Overmann
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE for details.

use ass6510::{assemble_file, disassemble, hexdump, opcode_list, Assembly, Diagnostic};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

/// The selected action for the command-line invocation.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Action {
    #[default]
    Assemble,
    Disassemble,
    DisassembleSource,
    Hexdump8,
    Hexdump16,
    OpcodeList,
    MakeModule,
    MakePlain,
}

/// Parsed command-line options.
#[derive(Default)]
struct Options {
    /// Selected assembler, disassembler, or conversion operation.
    action: Action,
    /// Source or object input filename.
    input: Option<PathBuf>,
    /// Object output filename for assembly.
    output: Option<PathBuf>,
    /// Optional symbol listing filename.
    symbols: Option<PathBuf>,
    /// Whether object input and output include a two-byte load address.
    module: bool,
    /// Whether to display assembler warnings.
    warnings: bool,
    /// Whether to show individual file operations.
    verbose: bool,
    /// Whether to validate without writing files.
    dry_run: bool,
    /// Whether to limit printed errors to twenty.
    stop_on_errors: bool,
    /// Whether to disable DSB compression in disassembly.
    no_dsb: bool,
    /// Whether to disable NOP compression in disassembly.
    no_nop: bool,
    /// Whether to display unsupported opcodes as ???.
    illegal: bool,
    /// Start address for headerless object input.
    start_address: u16,
    /// New address for make-module.
    module_address: u16,
}

/// Print a short command-line reference.
fn help() {
    print!(
        "A simple 6510 assembler and object-code utility.\n\n\
Usage: ass6510 [OPTIONS] <infile>\n\n\
Assembler options:\n\
  -m, --module             Prefix output with the load address (little endian)\n\
  -o, --output-file FILE   Set object output path (default: <infile>.6510)\n\
  -s, --symbol-file FILE   Write a symbol listing\n\
  -w, --warnings           Print assembler warnings\n\
  -f, --stop-on-errors     Show at most 20 errors\n\n\
Object-code options:\n\
  -d, --disassemble       Print a disassembly\n\
  -S, --disass-source      Print reassemblable source\n\
  -D, --hexdump            Print an 8-byte-wide hex dump\n\
  -W, --wide-hexdump       Print a 16-byte-wide hex dump\n\
      --opcode-list        Print all 256 opcodes\n\
      --no-dsb             Do not compress repeated zero bytes\n\
      --no-nop             Do not compress repeated NOP bytes\n\
      --illegal            Display unsupported opcodes as ???\n\
  -O, --start-address ADR  Base address for headerless input\n\
  -M, --make-module ADR    Add or replace a two-byte load address\n\
  -P, --make-plain         Remove a two-byte load address\n\n\
Common options:\n\
  -v, --verbose            Show individual operations\n\
      --dry-run            Validate without writing files\n\
  -h, --help               Show this help\n\
      --version            Show version\n\n\
ass6510 version {} ({})\n\
Copyright (c) 2026 Johannes Overmann\n\
https://github.com/jovermann/ass6510\n",
        env!("CARGO_PKG_VERSION"),
        option_env!("BUILD_DATE").unwrap_or("build date unavailable")
    );
}

/// Parse a command-line address in decimal or assembler notation.
fn parse_address(text: &str) -> Result<u16, String> {
    let (digits, radix) = if let Some(value) = text.strip_prefix('$') {
        (value, 16)
    } else if let Some(value) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        (value, 16)
    } else {
        (text, 10)
    };
    let value =
        u32::from_str_radix(digits, radix).map_err(|_| format!("invalid address '{text}'"))?;
    u16::try_from(value).map_err(|_| format!("address out of range: {text}"))
}

/// Consume an option's separate or equals-style value.
fn option_value(args: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    if let Some((_, value)) = args[*index].split_once('=') {
        return Ok(value.to_owned());
    }
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("{option} needs a value"))
}

/// Parse the supported short and long options.
fn parse_options(args: &[String]) -> Result<Option<Options>, String> {
    let mut options = Options::default();
    let mut index = 1;
    let mut positional = false;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            positional = true;
            index += 1;
            continue;
        }
        if !positional {
            match arg.as_str() {
                "-h" | "--help" => {
                    help();
                    return Ok(None);
                }
                "--version" => {
                    println!("ass6510 {}", env!("CARGO_PKG_VERSION"));
                    return Ok(None);
                }
                "-m" | "--module" => options.module = true,
                "-w" | "--warnings" => options.warnings = true,
                "-f" | "--stop-on-errors" => options.stop_on_errors = true,
                "-v" | "--verbose" => options.verbose = true,
                "--dry-run" => options.dry_run = true,
                "--no-dsb" => options.no_dsb = true,
                "--no-nop" => options.no_nop = true,
                "--illegal" => options.illegal = true,
                "-d" | "--disassemble" => options.action = Action::Disassemble,
                "-S" | "--disass-source" => options.action = Action::DisassembleSource,
                "-D" | "--hexdump" => options.action = Action::Hexdump8,
                "-W" | "--wide-hexdump" => options.action = Action::Hexdump16,
                "--opcode-list" => options.action = Action::OpcodeList,
                "-P" | "--make-plain" => options.action = Action::MakePlain,
                _ if arg == "-o" || arg.starts_with("--output-file=") || arg == "--output-file" => {
                    options.output = Some(option_value(args, &mut index, "--output-file")?.into());
                }
                _ if arg == "-s" || arg.starts_with("--symbol-file=") || arg == "--symbol-file" => {
                    options.symbols = Some(option_value(args, &mut index, "--symbol-file")?.into());
                }
                _ if arg == "-O"
                    || arg.starts_with("--start-address=")
                    || arg == "--start-address" =>
                {
                    options.start_address =
                        parse_address(&option_value(args, &mut index, "--start-address")?)?;
                }
                _ if arg == "-M" || arg.starts_with("--make-module=") || arg == "--make-module" => {
                    options.module_address =
                        parse_address(&option_value(args, &mut index, "--make-module")?)?;
                    options.action = Action::MakeModule;
                }
                _ if arg.starts_with('-') => return Err(format!("unknown option '{arg}'")),
                _ => {
                    if options.input.replace(arg.into()).is_some() {
                        return Err("need exactly one input file".into());
                    }
                }
            }
        } else if options.input.replace(arg.into()).is_some() {
            return Err("need exactly one input file".into());
        }
        index += 1;
    }
    if options.action != Action::OpcodeList && options.input.is_none() {
        return Err("need exactly one input file".into());
    }
    Ok(Some(options))
}

/// Read object code and its optional two-byte load address.
fn load_object(path: &PathBuf, module: bool, start_address: u16) -> Result<(Vec<u8>, u16), String> {
    let mut bytes =
        fs::read(path).map_err(|error| format!("cannot read '{}': {error}", path.display()))?;
    let base = if module {
        if bytes.len() < 2 {
            return Err("module is missing its two-byte load address".into());
        }
        let base = u16::from_le_bytes([bytes[0], bytes[1]]);
        bytes.drain(..2);
        base
    } else {
        start_address
    };
    if base as usize + bytes.len() > 65_536 {
        return Err("object code exceeds the 16-bit address space".into());
    }
    Ok((bytes, base))
}

/// Write the assembled bytes and an optional module header.
fn save_object(
    path: &PathBuf,
    assembly: &Assembly,
    module: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<(), String> {
    let mut bytes = Vec::with_capacity(assembly.bytes.len() + if module { 2 } else { 0 });
    if module {
        bytes.extend_from_slice(&assembly.base.to_le_bytes());
    }
    bytes.extend_from_slice(&assembly.bytes);
    if verbose || dry_run {
        eprintln!(
            "{} {} bytes to '{}'",
            if dry_run { "would write" } else { "writing" },
            bytes.len(),
            path.display()
        );
    }
    if !dry_run {
        fs::write(path, bytes)
            .map_err(|error| format!("cannot write '{}': {error}", path.display()))?;
    }
    Ok(())
}

/// Format global symbols for a symbol listing.
fn symbol_listing(assembly: &Assembly) -> String {
    let width = assembly.symbols.keys().map(String::len).max().unwrap_or(0);
    let mut out = format!("list of {} global symbols:\n", assembly.symbols.len());
    for (name, symbol) in &assembly.symbols {
        let unsigned = symbol.value as u16;
        let signed = unsigned as i16;
        let location = symbol.file.as_ref().map_or_else(
            || format!("line {}", symbol.line),
            |file| format!("{}:{}", file.display(), symbol.line),
        );
        out.push_str(&format!("{name:width$} = ${unsigned:04X} == {unsigned:5} == {signed:6} defined at {location} {}\n",
            if symbol.used { "" } else { "(unused)" }));
    }
    out
}

/// Format a source diagnostic with its original file and line.
fn diagnostic_location(diagnostic: &Diagnostic) -> String {
    diagnostic.file.as_ref().map_or_else(
        || format!("line {}", diagnostic.line),
        |file| format!("{}:{}", file.display(), diagnostic.line),
    )
}

/// Execute the selected assembler or object-code operation.
fn run(options: Options) -> Result<(), String> {
    if options.action == Action::OpcodeList {
        print!("{}", opcode_list());
        return Ok(());
    }
    let input = options.input.as_ref().unwrap();
    if options.verbose {
        eprintln!("reading '{}'", input.display());
    }
    match options.action {
        Action::Assemble => {
            let assembly = match assemble_file(input) {
                Ok(assembly) => assembly,
                Err(errors) => {
                    let count = if options.stop_on_errors {
                        20
                    } else {
                        usize::MAX
                    };
                    for diagnostic in errors.iter().take(count) {
                        eprintln!(
                            "error ({}): {}",
                            diagnostic_location(diagnostic),
                            diagnostic.message
                        );
                    }
                    return Err(format!("assembly failed with {} error(s)", errors.len()));
                }
            };
            if options.warnings {
                for diagnostic in &assembly.warnings {
                    eprintln!(
                        "warning ({}): {}",
                        diagnostic_location(diagnostic),
                        diagnostic.message
                    );
                }
            }
            let output = options
                .output
                .unwrap_or_else(|| PathBuf::from(format!("{}.6510", input.display())));
            save_object(
                &output,
                &assembly,
                options.module,
                options.dry_run,
                options.verbose,
            )?;
            if let Some(path) = options.symbols {
                let text = symbol_listing(&assembly);
                if options.verbose || options.dry_run {
                    eprintln!(
                        "{} symbols to '{}'",
                        if options.dry_run {
                            "would write"
                        } else {
                            "writing"
                        },
                        path.display()
                    );
                }
                if !options.dry_run {
                    fs::write(&path, text)
                        .map_err(|error| format!("cannot write '{}': {error}", path.display()))?;
                }
            }
            if options.verbose {
                eprintln!(
                    "assembled {} bytes at ${:04X}",
                    assembly.bytes.len(),
                    assembly.base
                );
            }
        }
        Action::MakeModule | Action::MakePlain => {
            let (bytes, _) = load_object(
                input,
                options.action == Action::MakePlain || options.module,
                0,
            )?;
            let mut output = Vec::new();
            if options.action == Action::MakeModule {
                output.extend_from_slice(&options.module_address.to_le_bytes());
            }
            output.extend_from_slice(&bytes);
            if options.verbose || options.dry_run {
                eprintln!(
                    "{} {} bytes to '{}'",
                    if options.dry_run {
                        "would write"
                    } else {
                        "writing"
                    },
                    output.len(),
                    input.display()
                );
            }
            if !options.dry_run {
                fs::write(input, output)
                    .map_err(|error| format!("cannot write '{}': {error}", input.display()))?;
            }
        }
        Action::Disassemble | Action::DisassembleSource | Action::Hexdump8 | Action::Hexdump16 => {
            let (bytes, base) = load_object(input, options.module, options.start_address)?;
            if options.verbose {
                eprintln!("loaded {} bytes at ${base:04X}", bytes.len());
            }
            let output = match options.action {
                Action::Disassemble => disassemble(
                    &bytes,
                    base,
                    false,
                    options.no_dsb,
                    options.no_nop,
                    options.illegal,
                ),
                Action::DisassembleSource => disassemble(
                    &bytes,
                    base,
                    true,
                    options.no_dsb,
                    options.no_nop,
                    options.illegal,
                ),
                Action::Hexdump8 => hexdump(&bytes, base, 8),
                Action::Hexdump16 => hexdump(&bytes, base, 16),
                _ => unreachable!(),
            };
            print!("{output}");
        }
        Action::OpcodeList => unreachable!(),
    }
    Ok(())
}

/// Parse command-line arguments and return a conventional process status.
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match parse_options(&args).and_then(|options| options.map(run).unwrap_or(Ok(()))) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("ass6510: {message}");
            ExitCode::FAILURE
        }
    }
}
