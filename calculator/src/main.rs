use calculator::Compile;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "vm")] {
        use calculator::compiler::vm::vm::VM as Engine;
    } else if #[cfg(feature = "jit")] {
        use calculator::compiler::jit::Jit as Engine;
    } else {
        use calculator::compiler::interpreter::Interpreter as Engine;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: calculator <filename>");
        std::process::exit(1);
    }
    println!(
        "{:?}",
        Engine::from_source(std::fs::read_to_string(&args[1]).unwrap().as_str()).unwrap()
    )
}
