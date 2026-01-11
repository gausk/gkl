#[cfg(feature = "interpreter")]
pub mod interpreter;

#[cfg(feature = "jit")]
pub mod jit;
#[cfg(feature = "vm")]
pub mod vm;
// Interpreter — Executes source (or its AST) directly by evaluating it step-by-step at runtime.

// JIT (Just-In-Time Compiler) — Compiles code to machine code at runtime, then executes the compiled result immediately.

// AOT (Ahead-Of-Time Compiler) — Compiles code to machine code before execution, producing a binary that runs later.

// VM (Virtual Machine) — Runs programs inside a simulated execution environment, usually executing bytecode instead of native machine code.
