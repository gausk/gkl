use anyhow::{Result, anyhow};
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;

/// Convenience type alias for the `add` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type Addition = unsafe extern "C" fn(u32, u32) -> u32;

struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    engine: ExecutionEngine<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> CodeGenerator<'ctx> {
    fn jit_compile_add(&self) -> Option<JitFunction<'_, Addition>> {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let fn_val = self.module.add_function("add", fn_type, None);
        let entry_block = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry_block);

        let x = fn_val.get_nth_param(0).unwrap().into_int_value();
        let y = fn_val.get_nth_param(1).unwrap().into_int_value();
        let ret = self.builder.build_int_add(x, y, "add").unwrap();
        self.builder.build_return(Some(&ret)).unwrap();

        unsafe { self.engine.get_function("add").ok() }
    }
}

fn main() -> Result<()> {
    let context = Context::create();
    let module = context.create_module("addition");
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let codegen = CodeGenerator {
        context: &context,
        engine: execution_engine,
        module,
        builder: context.create_builder(),
    };
    let add = codegen
        .jit_compile_add()
        .ok_or_else(|| anyhow!("unable to JIT compile add"))?;

    println!("{:?}", add);

    let x = 1;
    let y = 2;

    unsafe {
        println!("{} + {} = {}", x, y, add.call(x, y));
        assert_eq!(add.call(x, y), x + y);
    }
    Ok(())
}
