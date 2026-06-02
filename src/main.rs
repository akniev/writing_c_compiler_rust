mod lexer_tokens;
mod lexer;
mod parser;
mod parser_tokens;

use clap::{ArgGroup, Parser};
use std::fs;

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
    }

    if args.parse {
        let tokens = lexer::tokenize(input_str);
        let program = parser::parse(tokens).unwrap();
        println!("{:?}", program);
    }
}

fn read_file(path: &str) -> String {
    let input_content = fs::read_to_string(path).expect("Failed to read input file");
    input_content
}