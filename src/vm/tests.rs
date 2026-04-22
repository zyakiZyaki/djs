#[cfg(test)]
mod tests {
    use crate::vm::opcode::{Chunk, OpCode};
    use crate::vm::compiler::Compiler;
    use crate::vm::vm::VM;
    use crate::values::Value;

    // ========================================================================
    // Chunk tests
    // ========================================================================

    #[test]
    fn test_chunk_new() {
        let chunk = Chunk::new();
        assert!(chunk.code.is_empty());
        assert!(chunk.constants.is_empty());
        assert!(chunk.properties.is_empty());
        assert!(chunk.globals.is_empty());
    }

    #[test]
    fn test_chunk_add_constant() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Number(42.0));
        let idx2 = chunk.add_constant(Value::Str("hello".into()));
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(chunk.constants.len(), 2);
    }

    #[test]
    fn test_chunk_emit() {
        let mut chunk = Chunk::new();
        chunk.emit(OpCode::PushConst(0));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        assert_eq!(chunk.code.len(), 3);
        assert_eq!(chunk.code[0], OpCode::PushConst(0));
        assert_eq!(chunk.code[1], OpCode::Add);
        assert_eq!(chunk.code[2], OpCode::Return);
    }

    #[test]
    fn test_chunk_patch_jump() {
        let mut chunk = Chunk::new();
        let jump_idx = chunk.code.len();
        chunk.emit(OpCode::JumpIfFalse(0));
        chunk.emit(OpCode::PushConst(0));
        let target = chunk.code.len();
        chunk.patch_jump(jump_idx, target);
        assert_eq!(chunk.code[jump_idx], OpCode::JumpIfFalse(target));
    }

    #[test]
    fn test_chunk_patch_jump_if_true() {
        let mut chunk = Chunk::new();
        let jump_idx = chunk.code.len();
        chunk.emit(OpCode::JumpIfTrue(0));
        let target = 99;
        chunk.patch_jump(jump_idx, target);
        assert_eq!(chunk.code[jump_idx], OpCode::JumpIfTrue(target));
    }

    #[test]
    fn test_chunk_patch_jump_unconditional() {
        let mut chunk = Chunk::new();
        let jump_idx = chunk.code.len();
        chunk.emit(OpCode::Jump(0));
        let target = 42;
        chunk.patch_jump(jump_idx, target);
        assert_eq!(chunk.code[jump_idx], OpCode::Jump(target));
    }

    #[test]
    fn test_chunk_add_global_dedup() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_global("foo");
        let idx2 = chunk.add_global("foo");
        let idx3 = chunk.add_global("bar");
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0); // deduplicated
        assert_eq!(idx3, 1);
        assert_eq!(chunk.globals.len(), 2);
    }

    #[test]
    fn test_chunk_add_property_dedup() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_property("length");
        let idx2 = chunk.add_property("length");
        let idx3 = chunk.add_property("size");
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0); // deduplicated
        assert_eq!(idx3, 1);
    }

    #[test]
    fn test_chunk_code_offset() {
        let mut chunk = Chunk::new();
        assert_eq!(chunk.code_offset(), 0);
        chunk.emit(OpCode::PushConst(0));
        assert_eq!(chunk.code_offset(), 1);
        chunk.emit(OpCode::Add);
        assert_eq!(chunk.code_offset(), 2);
    }

    #[test]
    fn test_chunk_disassemble() {
        let mut chunk = Chunk::new();
        chunk.emit(OpCode::PushConst(0));
        chunk.emit(OpCode::Add);
        chunk.emit(OpCode::Return);
        // Just verify it doesn't panic
        chunk.disassemble("test");
    }

    // ========================================================================
    // Compiler tests
    // ========================================================================

    #[test]
    fn test_compiler_compile_simple_expr() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("2 + 3");
        assert!(result.is_ok());
        let (chunk, _decls) = result.unwrap();
        assert!(!chunk.code.is_empty());
        assert!(matches!(chunk.code.last(), Some(OpCode::Return)));
    }

    #[test]
    fn test_compiler_compile_function_decl() {
        let mut compiler = Compiler::new();
        // Function declaration followed by a call expression
        let result = compiler.compile_program("function foo(x) {return x + 1}\nfoo(1)");
        assert!(result.is_ok());
        let (chunk, decls) = result.unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].name, "foo");
        assert!(!chunk.code.is_empty());
    }

    #[test]
    fn test_compiler_compile_conditional() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("1 ? 10 : 20");
        assert!(result.is_ok());
        let (chunk, _) = result.unwrap();
        let has_jump = chunk.code.iter().any(|op| matches!(op, OpCode::JumpIfFalsePop(_)));
        assert!(has_jump);
    }

    #[test]
    fn test_compiler_compile_comparison() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("5 > 3");
        assert!(result.is_ok());
        let (chunk, _) = result.unwrap();
        assert!(chunk.code.iter().any(|op| matches!(op, OpCode::Gt)));
    }

    #[test]
    fn test_compiler_compile_string() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("\"hello\"");
        assert!(result.is_ok());
        let (chunk, _) = result.unwrap();
        assert!(chunk.code.iter().any(|op| matches!(op, OpCode::PushConst(_))));
    }

    #[test]
    fn test_compiler_compile_array_literal() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("[1, 2, 3]");
        assert!(result.is_ok());
        let (chunk, _) = result.unwrap();
        assert!(chunk.code.iter().any(|op| matches!(op, OpCode::MakeArray)));
    }

    #[test]
    fn test_compiler_compile_object_literal() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("{name: \"max\"}");
        assert!(result.is_ok());
        let (chunk, _) = result.unwrap();
        assert!(chunk.code.iter().any(|op| matches!(op, OpCode::MakeObject(_))));
    }

    #[test]
    fn test_compiler_compile_nested_function() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("function outer(x) {function inner(y) {return y * 2} return inner(x) + 1}\nouter(5)");
        assert!(result.is_ok());
        assert_eq!(compiler.functions.len(), 2); // outer + inner
    }

    #[test]
    fn test_compiler_compile_closure_capture() {
        let mut compiler = Compiler::new();
        let result = compiler.compile_program("function make_adder(x) {function(y) {return x + y}}\nmake_adder(5)");
        assert!(result.is_ok());
        assert_eq!(compiler.functions.len(), 2); // make_adder + anonymous
        // The anonymous function should have captures (x from outer scope)
        // Just verify compilation succeeded and function count is correct
    }

    // ========================================================================
    // VM tests
    // ========================================================================

    #[test]
    fn test_vm_new() {
        let vm = VM::new();
        // VM should have JSON global - just verify construction works
        drop(vm);
    }

    #[test]
    fn test_vm_basic_arithmetic() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("2 + 3").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_vm_multiplication() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("3 * 7").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(21.0));
    }

    #[test]
    fn test_vm_division() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("15 / 3").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_vm_subtraction() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("10 - 4").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_vm_modulo() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("10 % 3").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_vm_conditional_true() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("1 ? 10 : 20").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_vm_conditional_false() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("0 ? 10 : 20").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_vm_comparison_eq() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("5 == 5").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_comparison_ne() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("5 != 3").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_comparison_lt() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("5 < 10").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_comparison_gt() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("5 > 10").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_string_literal() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("\"hello\"").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Str("hello".to_string()));
    }

    #[test]
    fn test_vm_array_literal() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("[1, 2, 3]").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)
        ]));
    }

    #[test]
    fn test_vm_empty_array() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("[]").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn test_vm_object_literal() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("{name: \"max\", age: 30}").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert!(matches!(result, Value::Object(_)));
        if let Value::Object(map) = result {
            assert_eq!(map.get("name"), Some(&Value::Str("max".to_string())));
            assert_eq!(map.get("age"), Some(&Value::Number(30.0)));
        }
    }

    #[test]
    fn test_vm_function_definition() {
        let mut compiler = Compiler::new();
        let source = "function foo(x) {return x + 1}\nfoo(1)";
        let (chunk, _) = compiler.compile_program(source).unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_vm_and_operator() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("1 && 1").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_or_operator() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("0 || 1").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_complex_arithmetic() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("2 + 3 * 4 - 1").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(13.0)); // 2 + 12 - 1
    }

    #[test]
    fn test_vm_parenthesized_expr() {
        let mut compiler = Compiler::new();
        let (chunk, _) = compiler.compile_program("(2 + 3) * 4").unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_vm_string_concatenation() {
        let mut compiler = Compiler::new();
        let source = "\"hello\" + \" \" + \"world\"";
        let (chunk, _) = compiler.compile_program(source).unwrap();
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Str("hello world".to_string()));
    }

    #[test]
    fn test_vm_compiled_function_count() {
        let mut compiler = Compiler::new();
        compiler.compile_program("1 + 1").unwrap();
        assert_eq!(compiler.functions.len(), 0);

        let mut compiler2 = Compiler::new();
        compiler2.compile_program("function foo() {return 1}\nfoo()").unwrap();
        assert_eq!(compiler2.functions.len(), 1);
    }

    #[test]
    fn test_vm_multiple_functions() {
        let mut compiler = Compiler::new();
        compiler.compile_program("function foo(x) {return x + 1}\nfoo(1)").unwrap();
        assert_eq!(compiler.functions.len(), 1);
        assert_eq!(compiler.functions[0].param_count, 1);
    }

    // ========================================================================
    // Module system tests
    // ========================================================================

    #[test]
    fn test_module_registry_new() {
        let registry = crate::vm::module::ModuleRegistry::new("/tmp");
        assert_eq!(registry.base_dir.to_str().unwrap(), "/tmp");
    }

    #[test]
    fn test_module_resolve_path_absolute() {
        use crate::vm::module::ModuleRegistry;
        let registry = ModuleRegistry::new("/test/base");
        let resolved = registry.resolve_path("/absolute/path.js");
        assert_eq!(resolved.to_str().unwrap(), "/absolute/path.js");
    }

    #[test]
    fn test_compile_program_multiple_functions() {
        // Test that compile_program handles multiple functions + expression
        let mut compiler = Compiler::new();
        let source = r#"
function add(a, b) { return a + b; }
function sub(a, b) { return a - b; }
add(10, sub(7, 2))
"#;
        let (chunk, decls) = compiler.compile_program(source).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(compiler.functions.len(), 2);
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(15.0)); // 10 + (7-2) = 15
    }

    #[test]
    fn test_compile_program_single_function() {
        let mut compiler = Compiler::new();
        let (chunk, decls) = compiler.compile_program("function fib(n) {return n > 1 ? fib(n-1) + fib(n-2) : n}\nfib(5)").unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].name, "fib");
        let mut vm = VM::new();
        let result = vm.run(chunk, compiler.functions).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }
}
