mod lexer;
mod parser;
mod codegen;

use lexer::Lexer;
use parser::Parser;
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("usage: gaufre <fichier.gfr> [out.wat]");
    let out_path = env::args().nth(2);

    let src = fs::read_to_string(&path)?;
    let lx = Lexer::new(&src);
    let mut p = Parser::new(lx)?;
    let ast = p.parse_program()?; // Program { logs }

    let wat = codegen::generate_wat(&ast);

    // Écrit soit vers 2e argument, soit <input>.wat, soit stdout si input == "-"
    if path == "-" {
        // lecture stdin / écriture stdout
        print!("{wat}");
    } else {
        let default_out = Path::new(&path)
            .with_extension("wat")
            .to_string_lossy()
            .into_owned();
        let out = out_path.unwrap_or(default_out);
        fs::write(&out, wat)?;
        eprintln!("Écrit: {}", out);
    }

    Ok(())
}
