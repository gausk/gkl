use calculator::Compile;
use cfg_if::cfg_if;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

cfg_if! {
    if #[cfg(feature = "vm")] {
        use calculator::compiler::vm::vm::VM as Engine;
    } else if #[cfg(feature = "jit")] {
        use calculator::compiler::jit::Jit as Engine;
    } else {
        use calculator::compiler::interpreter::Interpreter as Engine;
    }
}

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                println!("{:?}", Engine::from_source(&line));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
