use wasmer::{FunctionMiddleware, MiddlewareError, MiddlewareReaderState, ModuleMiddleware};
use wasmer::wasmparser::Operator;
use wasmer_types::{LocalFunctionIndex, ModuleInfo};

#[derive(Debug, loupe::MemoryUsage)]
pub struct ModuleTransformer {
    
}

#[derive(Debug, loupe::MemoryUsage)]
pub struct FunctionTransformer {
    fn_id: u32,
}

impl Default for ModuleTransformer {
    fn default() -> Self {
        Self {}
    }
}

impl ModuleMiddleware for ModuleTransformer {
    fn generate_function_middleware(&self, _lfi: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionTransformer {
            fn_id: 0,
        })
    }

    fn transform_module_info(&self, _info: &mut ModuleInfo) {}
}

impl FunctionMiddleware for FunctionTransformer {
    fn feed<'a>(&mut self, operator: Operator<'a>, state: &mut MiddlewareReaderState<'a>) -> Result<(), MiddlewareError> {
        // if let Operator::LocalGet { .. } = &operator {
        //     state.push_operator(Operator::Call { function_index: 0 })
        // }
        state.push_operator(operator);
        Ok(())
    }
}
