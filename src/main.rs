mod lexer_tokens;
mod lexer;
mod parser;
mod parser_tokens;
mod assembly;
mod assembly_tokens;
mod code_emission;
mod tacky_tokens;
mod tacky;
mod fix_pseudo;
mod fix_mov;

use clap::{ArgGroup, Parser};
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::fix_mov::fix_movs_in_program;
use crate::fix_pseudo::fix_pseudo_in_program;

#[derive(Parser, Debug)]
#[command(about = "C compiler arguments parser")]
#[command(group(
ArgGroup::new("stage")
    .args(&["lex", "parse", "validate", "codegen", "tacky", "compile"])
    .multiple(false)
))]
struct Args {
    #[arg(long)]
    lex: bool,

    #[arg(long)]
    parse: bool,

    #[arg(long)]
    validate: bool,

    #[arg(long)]
    codegen: bool,

    #[arg(long)]
    tacky: bool,

    #[arg(long)]
    compile: bool,

    #[arg(long)]
    c: bool,

    input: String,
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");
    let input = read_file(&args.input);
    let input_str = input.as_str();

    if args.lex {
        let tokens = lexer::tokenize(input_str);
        println!("{:?}", tokens);
    } else if args.parse {
        let tokens = lexer::tokenize(input_str);
        let program = parser::parse(tokens).unwrap();
        println!("{:?}", program);
    } else if args.tacky {
        let tokens = lexer::tokenize(input_str);
        let program = parser::parse(tokens).unwrap();
        let tacky = tacky::parse_tacky(program).unwrap();
        println!("{:?}", tacky);
    } else if args.codegen {
        let tokens = lexer::tokenize(input_str);
        let program = parser::parse(tokens).unwrap();
        let tacky = tacky::parse_tacky(program).unwrap();
        let assembly = assembly::generate_assembly(tacky);
        let assembly_pseudo_fixed = fix_pseudo_in_program(assembly);
        let assembly_mov_fixed = fix_movs_in_program(assembly_pseudo_fixed);
        println!("{:?}", assembly_mov_fixed);
    } else {
        let tokens = lexer::tokenize(input_str);
        let program = parser::parse(tokens).unwrap();
        let tacky = tacky::parse_tacky(program).unwrap();
        let assembly = assembly::generate_assembly(tacky);
        let assembly_pseudo_fixed = fix_pseudo_in_program(assembly);
        let assembly_mov_fixed = fix_movs_in_program(assembly_pseudo_fixed);
        let code = code_emission::generate_code(assembly_mov_fixed);
        println!("{}", code);

        let path = Path::new(&args.input);
        let asm_file = path.with_extension("s");
        let out_file = path.with_extension("");

        fs::write(&asm_file, code).expect("Failed to write assembly file");

        let output = if args.c {
            Command::new("gcc")
                .args(["-c"])
                .arg(&asm_file)
                .arg("-o")
                .arg(&out_file.with_extension("o"))
                .output()
        } else {
            Command::new("gcc")
                .arg(&asm_file)
                .arg("-o")
            .arg(&out_file)
            .output()
        }.expect("Failed to execute command");

        if output.status.success() {
            println!("Compilation successful");
        } else {
            println!("Compilation failed");
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }
}

fn read_file(path: &str) -> String {
    let input_content = fs::read_to_string(path).expect("Failed to read input file");
    input_content
}