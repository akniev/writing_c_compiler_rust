use crate::tacky::tacky_tokens::TProgram;

pub fn print_tacky(program: &TProgram) -> String {
    format!("{:#?}", program)
}