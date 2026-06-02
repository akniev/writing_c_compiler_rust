mod token;
mod lexer;

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
    let input_content = fs::read_to_string(args.input).expect("Failed to read input file");
    let input = input_content.as_str();
    let tokens = lexer::tokenize(input);
    println!("{:?}", tokens);
}
