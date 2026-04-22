// Bytecode Virtual Machine — optimized for functional language
// Architecture: flat locals per frame, pre-captured closure values, proper param handling

use crate::vm::opcode::*;
use crate::values::Value;
use crate::parser::BinOp;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BytecodeClosure {
    pub fn_idx: usize,
    pub captured: Vec<Value>,
    pub nesting_depth: usize,
}

pub(crate) struct CallFrame {
    fn_idx: usize,
    ip: usize,
    locals: Vec<Option<Value>>,
}

pub struct VM {
    value_stack: Vec<Value>,
    frames: Vec<CallFrame>,
    functions: Vec<CompiledFunction>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("JSON".to_string(), Value::Object(HashMap::new()));

        // console object
        let mut console_obj = HashMap::new();
        console_obj.insert("log".to_string(), Value::Builtin("console.log".to_string()));
        globals.insert("console".to_string(), Value::Object(console_obj));

        // Add fetch as a builtin
        globals.insert("fetch".to_string(), Value::Builtin("fetch".to_string()));

        // Test framework builtins
        globals.insert("test".to_string(), Value::Builtin("test".to_string()));
        globals.insert("expect".to_string(), Value::Builtin("expect".to_string()));
        globals.insert("describe".to_string(), Value::Builtin("describe".to_string()));

        // Create http module object with methods
        let mut http_obj = HashMap::new();
        http_obj.insert("get".to_string(), Value::Builtin("http.get".to_string()));
        http_obj.insert("post".to_string(), Value::Builtin("http.post".to_string()));
        http_obj.insert("put".to_string(), Value::Builtin("http.put".to_string()));
        http_obj.insert("delete".to_string(), Value::Builtin("http.delete".to_string()));
        http_obj.insert("patch".to_string(), Value::Builtin("http.patch".to_string()));
        http_obj.insert("request".to_string(), Value::Builtin("http.request".to_string()));
        http_obj.insert("createServer".to_string(), Value::Builtin("http.createServer".to_string()));
        http_obj.insert("listen".to_string(), Value::Builtin("http.listen".to_string()));
        globals.insert("http".to_string(), Value::Object(http_obj));
        
        VM {
            value_stack: Vec::with_capacity(2048),
            frames: Vec::with_capacity(128),
            functions: Vec::new(),
            globals,
        }
    }

    pub fn run(&mut self, chunk: Chunk, functions: Vec<CompiledFunction>) -> Result<Value, String> {
        self.functions = functions;
        let main_idx = self.functions.len();
        self.functions.push(CompiledFunction {
            chunk,
            param_count: 0,
            name: "<main>".to_string(),
            captured_slots: vec![],
            rest_slot: None,
            destructured_params: vec![],
            param_offset: 0,
            nesting_depth: 0,
        });
        self.frames.push(CallFrame {
            fn_idx: main_idx,
            ip: 0,
            locals: vec![None; 16],
        });
        self.execute()
    }

    fn execute(&mut self) -> Result<Value, String> {
        let max_steps = 500_000_000;
        let mut steps = 0u64;

        loop {
            steps += 1;
            if steps > max_steps {
                return Err("Execution limit exceeded".into());
            }

            let frame = self.frames.last_mut();
            let frame = match frame {
                Some(f) => f,
                None => return Ok(self.value_stack.pop().unwrap_or(Value::Number(0.0))),
            };

            let fn_idx = frame.fn_idx;
            let ip = frame.ip;

            let fn_data = &self.functions[fn_idx];
            let chunk = &fn_data.chunk;

            if ip >= chunk.code.len() {
                self.frames.pop();
                continue;
            }

            let op = chunk.code[ip].clone();
            frame.ip += 1;

            match op {
                OpCode::PushConst(idx) => {
                    self.value_stack.push(chunk.constants[idx as usize].clone());
                }
                OpCode::Dup => {
                    let v = self.value_stack.last().unwrap().clone();
                    self.value_stack.push(v);
                }
                OpCode::Pop => {
                    self.value_stack.pop();
                }
                OpCode::LoadLocal(slot) => {
                    let slot = slot as usize;
                    let frame = self.frames.last().unwrap();
                    match frame.locals.get(slot).and_then(|v| v.as_ref()).cloned() {
                        Some(v) => self.value_stack.push(v),
                        None => {
                            return Err(format!("Undefined local at slot {}", slot));
                        }
                    }
                }
                OpCode::StoreLocal(slot) => {
                    let val = self.value_stack.pop().unwrap();
                    let slot = slot as usize;
                    let frame = self.frames.last_mut().unwrap();
                    if slot >= frame.locals.len() {
                        frame.locals.resize(slot + 1, None);
                    }
                    frame.locals[slot] = Some(val.clone());
                    self.value_stack.push(val);
                }
                OpCode::LoadGlobal(idx) => {
                    let name = &chunk.globals[idx as usize];
                    match self.globals.get(name) {
                        Some(v) => self.value_stack.push(v.clone()),
                        None => return Err(format!("Undefined variable: {}", name)),
                    }
                }
                OpCode::StoreGlobal(idx) => {
                    let val = self.value_stack.pop().unwrap();
                    let name = chunk.globals[idx as usize].clone();
                    self.globals.insert(name, val);
                }
                OpCode::Add => self.bin_op(BinOp::Add)?,
                OpCode::Sub => self.bin_op(BinOp::Sub)?,
                OpCode::Mul => self.bin_op(BinOp::Mul)?,
                OpCode::Div => self.bin_op(BinOp::Div)?,
                OpCode::Mod => self.bin_op(BinOp::Mod)?,
                OpCode::Eq => {
                    let b = self.value_stack.pop().unwrap();
                    let a = self.value_stack.pop().unwrap();
                    self.value_stack.push(Value::Bool(a == b));
                }
                OpCode::Ne => {
                    let b = self.value_stack.pop().unwrap();
                    let a = self.value_stack.pop().unwrap();
                    self.value_stack.push(Value::Bool(a != b));
                }
                OpCode::Lt => self.bin_op(BinOp::Lt)?,
                OpCode::Gt => self.bin_op(BinOp::Gt)?,
                OpCode::Le => self.bin_op(BinOp::Le)?,
                OpCode::Ge => self.bin_op(BinOp::Ge)?,
                OpCode::Jump(t) => {
                    self.frames.last_mut().unwrap().ip = t;
                }
                OpCode::JumpIfFalse(t) => {
                    if crate::values::is_falsey(self.value_stack.last().unwrap()) {
                        self.frames.last_mut().unwrap().ip = t;
                    }
                }
                OpCode::JumpIfTrue(t) => {
                    if crate::is_truthy(self.value_stack.last().unwrap()) {
                        self.frames.last_mut().unwrap().ip = t;
                    }
                }
                OpCode::JumpIfFalsePop(t) => {
                    let v = self.value_stack.pop().unwrap();
                    if crate::values::is_falsey(&v) {
                        self.frames.last_mut().unwrap().ip = t;
                    }
                }
                OpCode::JumpIfTruePop(t) => {
                    let v = self.value_stack.pop().unwrap();
                    if crate::is_truthy(&v) {
                        self.frames.last_mut().unwrap().ip = t;
                    }
                }
                OpCode::ToBool => {
                    let v = self.value_stack.pop().unwrap();
                    self.value_stack.push(Value::Bool(crate::is_truthy(&v)));
                }
                OpCode::MakeObject(n) => {
                    let mut m = HashMap::new();
                    for _ in 0..n {
                        let v = self.value_stack.pop().unwrap();
                        let k = self.value_stack.pop().unwrap();
                        if let Value::Str(s) = k {
                            m.insert(s, v);
                        }
                    }
                    self.value_stack.push(Value::Object(m));
                }
                OpCode::Index => {
                    let idx = self.value_stack.pop().unwrap();
                    let base = self.value_stack.pop().unwrap();
                    self.value_stack.push(vm_index(&base, &idx)?);
                }
                OpCode::GetProperty(idx) => {
                    let prop = &chunk.properties[idx as usize];
                    let v = self.value_stack.pop().unwrap();
                    self.value_stack.push(vm_get_property(&v, prop)?);
                }
                OpCode::Spread => {
                    let v = self.value_stack.pop().unwrap();
                    if let Value::Array(items) = v {
                        for item in items {
                            self.value_stack.push(item);
                        }
                    } else {
                        return Err(format!("Cannot spread non-array: {:?}", v));
                    }
                }
                OpCode::PushDepthMarker => {
                    self.value_stack.push(Value::Number(f64::NEG_INFINITY));
                }
                OpCode::MakeArray => {
                    let mut elements = Vec::new();
                    while let Some(top) = self.value_stack.last() {
                        if let Value::Number(n) = top {
                            if n.is_infinite() && n.is_sign_negative() {
                                self.value_stack.pop();
                                break;
                            }
                        }
                        elements.push(self.value_stack.pop().unwrap());
                    }
                    elements.reverse();
                    self.value_stack.push(Value::Array(elements));
                }
                OpCode::MakeClosure(fn_idx) => {
                    let fn_idx = fn_idx as usize;
                    let fn_data = &self.functions[fn_idx];
                    let frame = self.frames.last().unwrap();
                    let captured: Vec<Value> = fn_data
                        .captured_slots
                        .iter()
                        .filter_map(|(parent_slot, _)| {
                            frame
                                .locals
                                .get(*parent_slot)
                                .and_then(|v| v.as_ref())
                                .cloned()
                        })
                        .collect();
                    self.value_stack.push(Value::BytecodeClosure(BytecodeClosure {
                        fn_idx,
                        captured,
                        nesting_depth: fn_data.nesting_depth,
                    }));
                }
                OpCode::Call(arg_count) => {
                    self.vm_call(arg_count as usize)?;
                }
                OpCode::CallSpread(_) => {
                    let mut closure_pos = 0;
                    for i in (0..self.value_stack.len()).rev() {
                        if matches!(self.value_stack[i], Value::BytecodeClosure(_)) {
                            closure_pos = i;
                            break;
                        }
                    }
                    let total_args = self.value_stack.len() - closure_pos - 1;
                    self.vm_call(total_args)?;
                }
                OpCode::TailCall(arg_count) => {
                    self.vm_call(arg_count as usize)?;
                }
                OpCode::TailCallSpread(_) => {
                    let mut closure_pos = 0;
                    for i in (0..self.value_stack.len()).rev() {
                        if matches!(self.value_stack[i], Value::BytecodeClosure(_)) {
                            closure_pos = i;
                            break;
                        }
                    }
                    let total_args = self.value_stack.len() - closure_pos - 1;
                    self.vm_call(total_args)?;
                }
                OpCode::MakePromise => {
                    let executor = self.value_stack.pop().unwrap();
                    let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
                        crate::values::PromiseState::new()
                    ));
                    let resolve = Value::Resolver(promise_state.clone());
                    let reject = Value::Rejector(promise_state.clone());
                    self.vm_call_callback(&executor, vec![resolve, reject])?;
                    self.value_stack.push(Value::Promise(promise_state));
                }
                OpCode::MethodCall(arg_count) => {
                    self.vm_method_call(arg_count)?;
                }
                OpCode::Return => {
                    let result = self.value_stack.pop().unwrap();
                    self.frames.pop();
                    if !self.frames.is_empty() {
                        self.value_stack.push(result);
                    } else {
                        return Ok(result);
                    }
                }
            }
        }
    }

    fn vm_call(&mut self, arg_count: usize) -> Result<(), String> {
        // Find closure on stack: it's at position (stack_len - arg_count - 1)
        let func_pos = self.value_stack.len() - arg_count - 1;

        let func_val = self.value_stack[func_pos].clone();
        
        // Handle resolve/reject specially
        if let Value::Resolver(p_state) = &func_val {
            let val = if arg_count > 0 {
                self.value_stack[func_pos + 1].clone()
            } else {
                Value::Bool(true)
            };
            let mut p = p_state.borrow_mut();
            if p.status == crate::values::PromiseStatus::Pending {
                p.status = crate::values::PromiseStatus::Fulfilled;
                p.value = Some(val);
            }
            self.value_stack.truncate(func_pos);
            self.value_stack.push(Value::Bool(true));
            return Ok(());
        }
        if let Value::Rejector(p_state) = &func_val {
            let val = if arg_count > 0 {
                self.value_stack[func_pos + 1].clone()
            } else {
                Value::Str("rejected".into())
            };
            let mut p = p_state.borrow_mut();
            if p.status == crate::values::PromiseStatus::Pending {
                p.status = crate::values::PromiseStatus::Rejected;
                p.value = Some(val);
            }
            self.value_stack.truncate(func_pos);
            self.value_stack.push(Value::Bool(true));
            return Ok(());
        }
        
        // Handle builtin functions like fetch()
        if let Value::Builtin(name) = &func_val {
            let args: Vec<Value> = self.value_stack.drain(func_pos + 1..).collect();
            self.value_stack.truncate(func_pos);
            let result = self.call_builtin(name, &args)?;
            self.value_stack.push(result);
            return Ok(());
        }
        
        match func_val {
            Value::Builtin(name) => {
                let args: Vec<Value> = self.value_stack.drain(func_pos + 1..).collect();
                self.value_stack.truncate(func_pos);
                let result = self.call_builtin(&name, &args)?;
                self.value_stack.push(result);
                return Ok(());
            }
            Value::BytecodeClosure(bc) => {
                let fn_data = &self.functions[bc.fn_idx];
                let num_params = fn_data.param_count;
                let offset = fn_data.param_offset;

                // Remove closure and args from stack
                let mut all: Vec<Value> = self.value_stack.drain(func_pos..).collect();
                let args: Vec<Value> = all.drain(1..).collect();

                // Calculate total param slots (including destructured fields)
                let mut total_param_slots = num_params;
                for (param_idx, fields) in &fn_data.destructured_params {
                    // Destructured param needs fields.len() slots (replaces 1 param slot)
                    total_param_slots = total_param_slots - 1 + fields.len();
                }

                let captured_base = offset + total_param_slots;
                let total_slots = (captured_base + bc.captured.len()).max(16);
                let mut locals = vec![None; total_slots];

                // Slot 0 = self for recursion (named functions)
                if offset > 0 {
                    locals[0] = Some(Value::BytecodeClosure(BytecodeClosure {
                        fn_idx: bc.fn_idx,
                        captured: bc.captured.clone(),
                        nesting_depth: bc.nesting_depth,
                    }));
                }

                // Place arguments and handle destructuring/rest
                let mut arg_idx = 0;
                let mut param_slot = offset;

                for i in 0..num_params {
                    // Check if this param is destructured
                    if let Some((_, fields)) = fn_data
                        .destructured_params
                        .iter()
                        .find(|(param_idx, _)| *param_idx == i)
                    {
                        // Place the object at this slot
                        if arg_idx < args.len() {
                            let obj = args[arg_idx].clone();
                            locals[param_slot] = Some(obj.clone());

                            // Extract fields into subsequent slots
                            if let Value::Object(map) = &obj {
                                for (j, field) in fields.iter().enumerate() {
                                    if let Some(val) = map.get(field) {
                                        locals[param_slot + j] = Some(val.clone());
                                    }
                                }
                            }
                            arg_idx += 1;
                        }
                        param_slot += fields.len();
                    } else if fn_data.rest_slot == Some(i) {
                        // Rest param — collect all remaining args
                        let remaining: Vec<Value> = args[arg_idx..].to_vec();
                        locals[param_slot] = Some(Value::Array(remaining));
                        arg_idx = args.len();
                        param_slot += 1;
                    } else {
                        // Regular param
                        if arg_idx < args.len() {
                            locals[param_slot] = Some(args[arg_idx].clone());
                            arg_idx += 1;
                        }
                        param_slot += 1;
                    }
                }

                // Place captured values after all params
                for (i, val) in bc.captured.iter().enumerate() {
                    let slot = captured_base + i;
                    if slot < locals.len() {
                        locals[slot] = Some(val.clone());
                    }
                }

                self.frames.push(CallFrame {
                    fn_idx: bc.fn_idx,
                    ip: 0,
                    locals,
                });
            }
            _ => {
                return Err("Attempted to call non-function".into());
            }
        }
        Ok(())
    }

    fn vm_method_call(&mut self, arg_count: u16) -> Result<(), String> {
        let args_start = self.value_stack.len() - arg_count as usize;
        let method_name = self.value_stack[args_start - 1].clone();
        let obj = self.value_stack[args_start - 2].clone();

        let method_str = match method_name {
            Value::Str(s) => s,
            _ => {
                self.value_stack.truncate(args_start - 2);
                return Err("Method name must be string".into());
            }
        };

        let result = self.exec_method(&obj, &method_str, arg_count, args_start)?;

        // If result is Number(0.0), exec_method already handled stack and left result on it
        if matches!(result, Value::Number(n) if n == 0.0) {
            return Ok(());
        }

        // For builtin methods, truncate and push the result
        self.value_stack.truncate(args_start - 2);
        self.value_stack.push(result);
        Ok(())
    }

    fn exec_method(
        &mut self,
        obj: &Value,
        method: &str,
        arg_count: u16,
        args_start: usize,
    ) -> Result<Value, String> {
        // Check if object has a closure property with this name
        if let Value::Object(map) = obj {
            if let Some(method_val) = map.get(method) {
                if let Value::BytecodeClosure(bc) = method_val {
                    // Collect arguments
                    let args: Vec<Value> = (0..arg_count)
                        .map(|i| self.value_stack[args_start + i as usize].clone())
                        .collect();
                    
                    // Set up the call frame manually
                    let fn_data = &self.functions[bc.fn_idx];
                    let num_params = fn_data.param_count;
                    let offset = fn_data.param_offset;

                    // Calculate total param slots
                    let mut total_param_slots = num_params;
                    for (_, fields) in &fn_data.destructured_params {
                        total_param_slots = total_param_slots - 1 + fields.len();
                    }

                    let captured_base = offset + total_param_slots;
                    let total_slots = (captured_base + bc.captured.len()).max(16);
                    let mut locals = vec![None; total_slots];

                    // Slot 0 = self for recursion
                    if offset > 0 {
                        locals[0] = Some(Value::BytecodeClosure(BytecodeClosure {
                            fn_idx: bc.fn_idx,
                            captured: bc.captured.clone(),
                            nesting_depth: bc.nesting_depth,
                        }));
                    }

                    // Place arguments
                    for i in 0..args.len().min(num_params) {
                        locals[offset + i] = Some(args[i].clone());
                    }

                    // Place captured values
                    for (i, val) in bc.captured.iter().enumerate() {
                        locals[captured_base + i] = Some(val.clone());
                    }

                    // Remove obj, method name, and args from stack
                    // The result will be pushed by the RETURN opcode
                    let stack_after_args = args_start + arg_count as usize;
                    self.value_stack.truncate(stack_after_args);

                    // Push the frame
                    self.frames.push(CallFrame {
                        fn_idx: bc.fn_idx,
                        ip: 0,
                        locals,
                    });
                    
                    // Execute until this frame returns
                    loop {
                        if self.frames.len() <= 1 {
                            return self.execute();
                        }
                        let frame_count = self.frames.len();
                        self.execute_one()?;
                        if self.frames.len() < frame_count {
                            break;
                        }
                    }
                    
                    // Result is now on the stack - return a dummy value
                    // vm_method_call will handle stack cleanup and return
                    return Ok(Value::Number(0.0));
                }
            }
        }

        // Object with builtin methods (like http.get, http.post, etc.)
        if let Value::Object(map) = obj {
            if let Some(Value::Builtin(builtin_name)) = map.get(method) {
                // Special case: http.listen needs the server object itself
                if builtin_name == "http.listen" {
                    if args_start > 1 {
                        let port_val = self.value_stack[args_start].clone();
                        let port = match port_val {
                            Value::Number(n) => n as u16,
                            _ => {
                                self.value_stack.truncate(args_start - 2);
                                return Err("listen() port must be a number".into());
                            }
                        };
                        
                        // Get handler from the server object (NOT globals)
                        let handler = match map.get("_handler") {
                            Some(h) => h.clone(),
                            None => {
                                self.value_stack.truncate(args_start - 2);
                                return Err("No handler — call http.createServer(handler) first".into());
                            }
                        };
                        
                        let functions = self.functions.clone();
                        let globals = self.globals.clone();
                        
                        let _ = crate::vm::builtins::start_server(port, move |req| {
                            if let Value::BytecodeClosure(bc) = &handler {
                                let req_value = crate::vm::builtins::req_to_value(req);
                                let mut req_vm = VM::new();
                                req_vm.functions = functions.clone();
                                req_vm.globals = globals.clone();
                                
                                let fn_data = &req_vm.functions[bc.fn_idx];
                                let offset = fn_data.param_offset;
                                let num_params = fn_data.param_count;
                                let mut total_param_slots = num_params;
                                for (_, fields) in &fn_data.destructured_params {
                                    total_param_slots = total_param_slots - 1 + fields.len();
                                }
                                let captured_base = offset + total_param_slots;
                                let total_slots = (captured_base + bc.captured.len()).max(16);
                                let mut locals = vec![None; total_slots];
                                
                                if offset > 0 { locals[0] = Some(handler.clone()); }
                                locals[offset] = Some(req_value.clone());
                                
                                for (i, val) in bc.captured.iter().enumerate() {
                                    let slot = captured_base + i;
                                    if slot < locals.len() { locals[slot] = Some(val.clone()); }
                                }
                                
                                req_vm.frames.push(crate::vm::vm::CallFrame {
                                    fn_idx: bc.fn_idx, ip: 0, locals,
                                });
                                
                                match req_vm.execute() {
                                    Ok(result) => {
                                        if let Value::Object(m) = &result {
                                            let status = m.get("status")
                                                .and_then(|v| match v { Value::Number(n) => Some(*n as u16), _ => None })
                                                .unwrap_or(200);
                                            let body = m.get("body")
                                                .and_then(|v| match v { Value::Str(s) => Some(s.clone()), _ => None })
                                                .unwrap_or_else(|| "Hello".to_string());
                                            let ct = m.get("headers")
                                                .and_then(|v| match v {
                                                    Value::Object(h) => h.get("Content-Type").and_then(|c| match c {
                                                        Value::Str(s) => Some(s.clone()), _ => None,
                                                    }), _ => None,
                                                })
                                                .unwrap_or_else(|| "text/plain".to_string());
                                            return (status, ct, body);
                                        }
                                        (200, "text/plain".to_string(), format_value_fb(&result))
                                    }
                                    Err(e) => (500, "text/plain".to_string(), format!("Server error: {}", e)),
                                }
                            } else {
                                (500, "text/plain".to_string(), "Handler is not a function".to_string())
                            }
                        });
                        
                        self.value_stack.truncate(args_start - 2);
                        self.value_stack.push(Value::Number(port as f64));
                        return Ok(Value::Number(port as f64));
                    }
                }
                
                // Regular builtin method call
                let args: Vec<Value> = self.value_stack[args_start..args_start + arg_count as usize].to_vec();
                return self.call_builtin(builtin_name, &args);
            }
        }

        // Array methods
        if let Value::Array(arr) = obj {
            match method {
                "map" => {
                    if arg_count < 1 {
                        return Err("map() requires a callback".into());
                    }
                    let cb = self.value_stack[args_start].clone();
                    let mut r = Vec::new();
                    for item in arr.iter() {
                        r.push(self.vm_call_callback(&cb, vec![item.clone()])?);
                    }
                    return Ok(Value::Array(r));
                }
                "filter" => {
                    if arg_count < 1 {
                        return Err("filter() requires a callback".into());
                    }
                    let cb = self.value_stack[args_start].clone();
                    let mut r = Vec::new();
                    for item in arr.iter() {
                        if crate::values::is_truthy(&self.vm_call_callback(&cb, vec![item.clone()])?) {
                            r.push(item.clone());
                        }
                    }
                    return Ok(Value::Array(r));
                }
                "reduce" => {
                    if arg_count < 1 {
                        return Err("reduce() requires a callback".into());
                    }
                    let cb = self.value_stack[args_start].clone();
                    let initial = if arg_count >= 2 {
                        self.value_stack[args_start + 1].clone()
                    } else if !arr.is_empty() {
                        arr.first().cloned().unwrap_or(Value::Number(0.0))
                    } else {
                        return Err("reduce() on empty array requires initial value".into());
                    };
                    let start = if arg_count < 2 { 1 } else { 0 };
                    let mut acc = initial;
                    for i in start..arr.len() {
                        acc = self.vm_call_callback(&cb, vec![acc, arr[i].clone()])?;
                    }
                    return Ok(acc);
                }
                "push" => {
                    let mut a = arr.clone();
                    for i in 0..arg_count as usize {
                        a.push(self.value_stack[args_start + i].clone());
                    }
                    return Ok(Value::Array(a));
                }
                "join" => {
                    let sep = if arg_count >= 1 {
                        match &self.value_stack[args_start] {
                            Value::Str(s) => s.clone(),
                            _ => ",".to_string(),
                        }
                    } else {
                        ",".to_string()
                    };
                    let parts: Vec<String> = arr.iter().map(format_value_for_join).collect();
                    return Ok(Value::Str(parts.join(&sep)));
                }
                "concat" => {
                    let mut r = arr.clone();
                    for i in 0..arg_count as usize {
                        let o = self.value_stack[args_start + i].clone();
                        if let Value::Array(items) = o {
                            r.extend(items);
                        }
                    }
                    return Ok(Value::Array(r));
                }
                "length" => return Ok(Value::Number(arr.len() as f64)),
                _ => {}
            }
        }

        // String methods
        if let Value::Str(s) = obj {
            match method {
                "length" => return Ok(Value::Number(s.len() as f64)),
                "split" => {
                    if arg_count >= 1 {
                        let sep = match &self.value_stack[args_start] {
                            Value::Str(sep_str) => sep_str.clone(),
                            _ => " ".to_string(),
                        };
                        let s = match obj {
                            Value::Str(s) => s.clone(),
                            _ => return Err("split() requires a string".into()),
                        };
                        return Ok(Value::Array(
                            s.split(&sep).map(|p| Value::Str(p.to_string())).collect(),
                        ));
                    }
                }
                _ => {}
            }
        }

        // Object methods
        if let Value::Object(map) = obj {
            match method {
                "keys" => {
                    return Ok(Value::Array(
                        map.keys().map(|k| Value::Str(k.clone())).collect(),
                    ));
                }
                "values" => {
                    return Ok(Value::Array(map.values().cloned().collect()));
                }
                _ => {}
            }
        }

        // JSON methods (empty object)
        if let Value::Object(map) = obj {
            if map.is_empty() {
                match method {
                    "parse" => {
                        if arg_count >= 1 {
                            let sv = self.value_stack[args_start].clone();
                            if let Value::Str(s) = sv {
                                return self.json_parse(&s);
                            }
                        }
                    }
                    "stringify" => {
                        if arg_count >= 1 {
                            let val = self.value_stack[args_start].clone();
                            return Ok(Value::Str(self.json_stringify(&val, 0)));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Promise support
        if matches!(obj, Value::Promise(_) | Value::Resolver(_) | Value::Rejector(_)) {
            match method {
                "then" => {
                    if arg_count >= 1 {
                        let cb = self.value_stack[args_start].clone();
                        let pval = match obj {
                            Value::Promise(state) => state.borrow().value.clone(),
                            _ => None,
                        };
                        if let Some(val) = pval {
                            let result = self.vm_call_callback(&cb, vec![val])?;
                            // Return a new promise fulfilled with callback result
                            let new_state = std::rc::Rc::new(std::cell::RefCell::new(
                                crate::values::PromiseState::new()
                            ));
                            new_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
                            new_state.borrow_mut().value = Some(result);
                            return Ok(Value::Promise(new_state));
                        }
                        return Err("Promise not fulfilled".into());
                    }
                }
                "catch" => {
                    if arg_count >= 1 {
                        let pval = match obj {
                            Value::Promise(state) => state.borrow().value.clone(),
                            _ => None,
                        };
                        if let Some(val) = pval {
                            return Ok(val);
                        }
                    }
                }
                "finally" => {
                    if arg_count >= 1 {
                        let cb = self.value_stack[args_start].clone();
                        let pval = match obj {
                            Value::Promise(state) => state.borrow().value.clone(),
                            _ => None,
                        };
                        if let Some(val) = pval {
                            self.vm_call_callback(&cb, vec![val.clone()]).ok();
                            // Return promise
                            let new_state = std::rc::Rc::new(std::cell::RefCell::new(
                                crate::values::PromiseState::new()
                            ));
                            new_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
                            new_state.borrow_mut().value = Some(val);
                            return Ok(Value::Promise(new_state));
                        }
                    }
                }
                _ => {}
            }
        }

        // Fetch Response methods
        if let Value::Object(map) = obj {
            if let Some(Value::Str(t)) = map.get("_type") {
                if t == "fetch_response" {
                    match method {
                        "json" => {
                            if let Some(Value::Str(body)) = map.get("_body_raw") {
                                // Parse JSON using serde
                                return match serde_json::from_str::<serde_json::Value>(body) {
                                    Ok(json) => Ok(self.json_to_value(&json)),
                                    Err(e) => Err(format!("Response.json(): {}", e)),
                                };
                            }
                            return Err("Response has no body".into());
                        }
                        "text" => {
                            if let Some(Value::Str(body)) = map.get("_body_raw") {
                                return Ok(Value::Str(body.clone()));
                            }
                            return Err("Response has no body".into());
                        }
                        _ => {}
                    }
                }
            }
        }

        // Expect methods (test framework)
        if let Value::Expect { value, negate } = obj {
            // Collect arguments from stack
            let args: Vec<Value> = (0..arg_count)
                .map(|i| self.value_stack[args_start + i as usize].clone())
                .collect();
            
            match method {
                "toEqual" => {
                    if args.is_empty() {
                        return Err("expect().toEqual() requires an argument".into());
                    }
                    let actual = &*value;
                    let expected = &args[0];
                    let equal = values_equal(actual, expected);
                    if *negate {
                        if equal {
                            return Err(format!("Expected values NOT to be equal, but both are {}", format_value_fb(actual)));
                        }
                        return Ok(Value::Bool(true));
                    }
                    if !equal {
                        return Err(format!("Expected {} but got {}", format_value_fb(expected), format_value_fb(actual)));
                    }
                    return Ok(Value::Bool(true));
                }
                "toBe" => {
                    if args.is_empty() {
                        return Err("expect().toBe() requires an argument".into());
                    }
                    let actual = &*value;
                    let expected = &args[0];
                    let equal = values_equal(actual, expected);
                    if *negate {
                        if equal {
                            return Err(format!("Expected values NOT to be equal, but both are {}", format_value_fb(actual)));
                        }
                        return Ok(Value::Bool(true));
                    }
                    if !equal {
                        return Err(format!("Expected {} but got {}", format_value_fb(expected), format_value_fb(actual)));
                    }
                    return Ok(Value::Bool(true));
                }
                "toBeTruthy" => {
                    let is_truthy = crate::values::is_truthy(&*value);
                    if *negate {
                        if is_truthy {
                            return Err(format!("Expected {} to be falsy but it's truthy", format_value_fb(&*value)));
                        }
                        return Ok(Value::Bool(true));
                    }
                    if !is_truthy {
                        return Err(format!("Expected {} to be truthy but it's falsy", format_value_fb(&*value)));
                    }
                    return Ok(Value::Bool(true));
                }
                "toBeFalsy" => {
                    let is_truthy = crate::values::is_truthy(&*value);
                    if *negate {
                        if !is_truthy {
                            return Err(format!("Expected {} to be truthy but it's falsy", format_value_fb(&*value)));
                        }
                        return Ok(Value::Bool(true));
                    }
                    if is_truthy {
                        return Err(format!("Expected {} to be falsy but it's truthy", format_value_fb(&*value)));
                    }
                    return Ok(Value::Bool(true));
                }
                "toBeGreaterThan" => {
                    if args.is_empty() {
                        return Err("expect().toBeGreaterThan() requires an argument".into());
                    }
                    let actual = match value.as_ref() {
                        Value::Number(a) => *a,
                        _ => return Err("toBeGreaterThan() requires numbers".into()),
                    };
                    let expected = match &args[0] {
                        Value::Number(e) => *e,
                        _ => return Err("toBeGreaterThan() requires numbers".into()),
                    };
                    if *negate {
                        if actual <= expected {
                            return Err(format!("Expected {} to NOT be greater than {}", actual, expected));
                        }
                        return Ok(Value::Bool(true));
                    }
                    if actual <= expected {
                        return Err(format!("Expected {} to be greater than {}", actual, expected));
                    }
                    return Ok(Value::Bool(true));
                }
                "toBeLessThan" => {
                    if args.is_empty() {
                        return Err("expect().toBeLessThan() requires an argument".into());
                    }
                    let actual = match value.as_ref() {
                        Value::Number(a) => *a,
                        _ => return Err("toBeLessThan() requires numbers".into()),
                    };
                    let expected = match &args[0] {
                        Value::Number(e) => *e,
                        _ => return Err("toBeLessThan() requires numbers".into()),
                    };
                    if *negate {
                        if actual >= expected {
                            return Err(format!("Expected {} to NOT be less than {}", actual, expected));
                        }
                        return Ok(Value::Bool(true));
                    }
                    if actual >= expected {
                        return Err(format!("Expected {} to be less than {}", actual, expected));
                    }
                    return Ok(Value::Bool(true));
                }
                "not" => {
                    return Ok(Value::Expect {
                        value: value.clone(),
                        negate: !negate,
                    });
                }
                _ => {}
            }
        }

        Err(format!("No method '{}' on {:?}", method, obj))
    }

    /// Call a builtin function by name with the given args
    fn call_builtin(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        match name {
            "fetch" => self.call_builtin_fetch(args),
            "console.log" => self.call_console_log(args),
            "http.get" => self.call_http_get(args),
            "http.post" => self.call_http_post(args),
            "http.put" => self.call_http_put(args),
            "http.delete" => self.call_http_delete(args),
            "http.patch" => self.call_http_patch(args),
            "http.request" => self.call_http_request(args),
            "http.createServer" => self.call_http_create_server(args),
            "test" => self.call_test(args),
            "expect" => self.call_expect(args),
            "describe" => self.call_describe(args),
            _ => Err(format!("Unknown builtin function: {}", name)),
        }
    }

    /// test(name, fn) — run a test function, catch errors
    fn call_test(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("test() requires a name argument".into());
        }
        let name = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => "test".to_string(),
        };

        if args.len() < 2 {
            eprintln!("⚠ {} — no test function provided", name);
            return Ok(Value::Bool(true));
        }

        let test_fn = &args[1];
        match test_fn {
            Value::BytecodeClosure(bc) => {
                // Execute the test function directly
                let result = self.vm_call_callback(test_fn, vec![]);
                match result {
                    Ok(_) => {
                        eprintln!("  ✓ {}", name);
                        Ok(Value::Bool(true))
                    }
                    Err(e) => {
                        eprintln!("  ✗ {}", name);
                        eprintln!("    Error: {}", e);
                        Err(format!("Test '{}' failed: {}", name, e))
                    }
                }
            }
            _ => Err("test() second argument must be a function".into()),
        }
    }

    /// expect(value) — return an expect object for chaining
    fn call_expect(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("expect() requires a value argument".into());
        }
        Ok(Value::Expect {
            value: Box::new(args[0].clone()),
            negate: false,
        })
    }

    /// describe(name, fn) — group tests
    fn call_describe(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("describe() requires a name argument".into());
        }
        let name = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => "describe".to_string(),
        };

        eprintln!("\n── {} ──", name);

        if args.len() >= 2 {
            match &args[1] {
                Value::BytecodeClosure(bc) => {
                    // Execute the describe block
                    let result = self.vm_call_callback(
                        &Value::BytecodeClosure(bc.clone()),
                        vec![]
                    );
                    if let Err(e) = result {
                        return Err(format!("Describe '{}' failed: {}", name, e));
                    }
                }
                _ => {}
            }
        }

        Ok(Value::Bool(true))
    }

    /// Helper: convert Value to JSON string
    fn value_to_json_string(&self, val: &Value) -> String {
        match val {
            Value::Number(n) => {
                if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{}", n) }
            }
            Value::Bool(true) => "true".into(),
            Value::Bool(false) => "false".into(),
            Value::Str(s) => format!("\"{}\"", s),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_json_string(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(map) => {
                let items: Vec<String> = map.iter().map(|(k, v)| {
                    format!("\"{}\": {}", k, self.value_to_json_string(v))
                }).collect();
                format!("{{{}}}", items.join(", "))
            }
            _ => "null".into(),
        }
    }

    /// Helper: convert serde_json::Value to our Value
    fn json_to_value(&self, json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Number(0.0),
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Value::Str(s.clone()),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.json_to_value(v)).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), self.json_to_value(v));
                }
                Value::Object(map)
            }
        }
    }

    /// Helper: convert Value to response object
    fn response_to_value(&self, response: &crate::vm::builtins::FetchResponse) -> Value {
        let mut resp_map = HashMap::new();
        resp_map.insert("status".to_string(), Value::Number(response.status as f64));
        resp_map.insert("statusText".to_string(), Value::Str(response.status_text.clone()));
        resp_map.insert("ok".to_string(), Value::Bool(response.ok));
        
        let mut headers_map = HashMap::new();
        for (k, v) in &response.headers {
            headers_map.insert(k.clone(), Value::Str(v.clone()));
        }
        resp_map.insert("headers".to_string(), Value::Object(headers_map));
        resp_map.insert("_type".to_string(), Value::Str("fetch_response".into()));
        resp_map.insert("_body_raw".to_string(), Value::Str(response.body.clone()));
        
        Value::Object(resp_map)
    }

    // ========================================================================
    // HTTP Module Methods (Node.js style)
    // ========================================================================

    /// http.get(url) → Promise<Response>
    fn call_http_get(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("http.get() requires a URL argument".into());
        }
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("http.get() URL must be a string".into()),
        };
        
        let response = crate::vm::builtins::http_get(&url)?;
        let resp_value = self.response_to_value(&response);
        
        // Return fulfilled Promise
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);
        Ok(Value::Promise(promise_state))
    }

    /// http.post(url, body) → Promise<Response>
    fn call_http_post(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("http.post() requires URL and body arguments".into());
        }
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("http.post() URL must be a string".into()),
        };
        let body = self.value_to_json_string(&args[1]);
        
        let response = crate::vm::builtins::http_post(&url, &body)?;
        let resp_value = self.response_to_value(&response);
        
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);
        Ok(Value::Promise(promise_state))
    }

    /// http.put(url, body) → Promise<Response>
    fn call_http_put(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("http.put() requires URL and body arguments".into());
        }
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("http.put() URL must be a string".into()),
        };
        let body = self.value_to_json_string(&args[1]);
        
        let response = crate::vm::builtins::http_put(&url, &body)?;
        let resp_value = self.response_to_value(&response);
        
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);
        Ok(Value::Promise(promise_state))
    }

    /// http.delete(url) → Promise<Response>
    fn call_http_delete(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("http.delete() requires a URL argument".into());
        }
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("http.delete() URL must be a string".into()),
        };
        
        let response = crate::vm::builtins::http_delete(&url)?;
        let resp_value = self.response_to_value(&response);
        
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);
        Ok(Value::Promise(promise_state))
    }

    /// http.patch(url, body) → Promise<Response>
    fn call_http_patch(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("http.patch() requires URL and body arguments".into());
        }
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("http.patch() URL must be a string".into()),
        };
        let body = self.value_to_json_string(&args[1]);
        
        let response = crate::vm::builtins::http_patch(&url, &body)?;
        let resp_value = self.response_to_value(&response);
        
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);
        Ok(Value::Promise(promise_state))
    }

    /// http.request(url, options) → Promise<Response>
    fn call_http_request(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("http.request() requires a URL argument".into());
        }
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("http.request() URL must be a string".into()),
        };
        
        let mut method = "GET".to_string();
        let mut headers = HashMap::new();
        let mut body: Option<String> = None;
        
        if args.len() > 1 {
            if let Value::Object(opts) = &args[1] {
                if let Some(Value::Str(m)) = opts.get("method") {
                    method = m.to_uppercase();
                }
                if let Some(Value::Object(h)) = opts.get("headers") {
                    for (k, v) in h {
                        if let Value::Str(vs) = v {
                            headers.insert(k.clone(), vs.clone());
                        }
                    }
                }
                if let Some(Value::Str(b)) = opts.get("body") {
                    body = Some(b.clone());
                } else if let Some(Value::Object(b)) = opts.get("body") {
                    body = Some(self.value_to_json_string(&Value::Object(b.clone())));
                }
            }
        }
        
        let response = crate::vm::builtins::http_request(&url, &method, &headers, body.as_deref())?;
        let resp_value = self.response_to_value(&response);
        
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);
        Ok(Value::Promise(promise_state))
    }

    // ========================================================================
    // HTTP Server
    // ========================================================================

    /// http.createServer(handler) — stores handler IN THE SERVER OBJECT
    fn call_http_create_server(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("http.createServer() requires a handler function".into());
        }
        
        if let Value::BytecodeClosure(bc) = &args[0] {
            if bc.nesting_depth == 0 {
                return Err("http.createServer() handler must be defined inside another function".into());
            }
        } else {
            return Err("http.createServer() handler must be a function".into());
        }
        
        let mut server_obj = HashMap::new();
        server_obj.insert("_handler".to_string(), args[0].clone());
        server_obj.insert("listen".to_string(), Value::Builtin("http.listen".to_string()));
        Ok(Value::Object(server_obj))
    }

    /// server.listen(port) — starts blocking HTTP server
    fn call_http_listen(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("listen() requires a port argument".into());
        }
        let port = match &args[0] {
            Value::Number(n) => *n as u16,
            _ => return Err("listen() port must be a number".into()),
        };
        
        // Get handler from the server object itself (NOT from globals)
        let handler = match self.globals.get("__http_server__") {
            Some(Value::Object(server)) => server.get("_handler").cloned(),
            _ => None,
        };
        
        let handler = match handler {
            Some(h) => h,
            None => return Err("No handler registered — call http.createServer(handler) first".into()),
        };
        
        let functions = self.functions.clone();
        let globals = self.globals.clone();
        
        // Start the server
        let _ = crate::vm::builtins::start_server(port, move |req| {
            if let Value::BytecodeClosure(bc) = &handler {
                let req_value = crate::vm::builtins::req_to_value(req);
                let mut req_vm = VM::new();
                req_vm.functions = functions.clone();
                req_vm.globals = globals.clone();
                
                let fn_data = &req_vm.functions[bc.fn_idx];
                let offset = fn_data.param_offset;
                let num_params = fn_data.param_count;
                let mut total_param_slots = num_params;
                for (_, fields) in &fn_data.destructured_params {
                    total_param_slots = total_param_slots - 1 + fields.len();
                }
                let captured_base = offset + total_param_slots;
                let total_slots = (captured_base + bc.captured.len()).max(16);
                let mut locals = vec![None; total_slots];
                
                if offset > 0 {
                    locals[0] = Some(handler.clone());
                }
                locals[offset] = Some(req_value.clone());
                
                for (i, val) in bc.captured.iter().enumerate() {
                    let slot = captured_base + i;
                    if slot < locals.len() {
                        locals[slot] = Some(val.clone());
                    }
                }
                
                req_vm.frames.push(crate::vm::vm::CallFrame {
                    fn_idx: bc.fn_idx,
                    ip: 0,
                    locals,
                });
                
                match req_vm.execute() {
                    Ok(result) => {
                        if let Value::Object(map) = &result {
                            let status = map.get("status")
                                .and_then(|v| match v { Value::Number(n) => Some(*n as u16), _ => None })
                                .unwrap_or(200);
                            
                            let body = map.get("body")
                                .and_then(|v| match v { Value::Str(s) => Some(s.clone()), _ => None })
                                .unwrap_or_else(|| "Hello".to_string());
                            
                            let content_type = map.get("headers")
                                .and_then(|v| match v {
                                    Value::Object(h) => h.get("Content-Type").and_then(|ct| match ct {
                                        Value::Str(s) => Some(s.clone()),
                                        _ => None,
                                    }),
                                    _ => None,
                                })
                                .unwrap_or_else(|| "text/plain".to_string());
                            
                            return (status, content_type, body);
                        }
                        (200, "text/plain".to_string(), format_value_fb(&result))
                    }
                    Err(e) => (500, "text/plain".to_string(), format!("Server error: {}", e)),
                }
            } else {
                (500, "text/plain".to_string(), "Handler is not a function".to_string())
            }
        });
        
        Ok(Value::Number(port as f64))
    }

    /// console.log(args...) — prints args to stderr and returns undefined
    fn call_console_log(&mut self, args: &[Value]) -> Result<Value, String> {
        let parts: Vec<String> = args.iter().map(format_value_fb).collect();
        eprintln!("{}", parts.join(" "));
        Ok(Value::Bool(true))
    }

    /// Implementation of fetch(url, options?)
    /// Returns a Promise that resolves with the Response
    fn call_builtin_fetch(&self, args: &[Value]) -> Result<Value, String> {
        use crate::vm::builtins::fetch;

        if args.is_empty() {
            return Err("fetch() requires at least a URL argument".into());
        }

        // Extract URL
        let url = match &args[0] {
            Value::Str(s) => s.clone(),
            _ => return Err("fetch() URL must be a string".into()),
        };

        // Extract options
        let mut method = "GET".to_string();
        let mut headers = HashMap::new();
        let mut body: Option<String> = None;

        if args.len() > 1 {
            if let Value::Object(opts) = &args[1] {
                if let Some(Value::Str(m)) = opts.get("method") {
                    method = m.to_uppercase();
                }
                if let Some(Value::Object(h)) = opts.get("headers") {
                    for (k, v) in h {
                        if let Value::Str(vs) = v {
                            headers.insert(k.clone(), vs.clone());
                        }
                    }
                }
                if let Some(Value::Str(b)) = opts.get("body") {
                    body = Some(b.clone());
                } else if let Some(Value::Object(b)) = opts.get("body") {
                    body = Some(self.value_to_json_string(&Value::Object(b.clone())));
                }
            }
        }

        // Execute fetch (blocking)
        let response = fetch(&url, &method, &headers, body.as_deref())?;
        
        // Build response object
        let mut resp_map = HashMap::new();
        resp_map.insert("status".to_string(), Value::Number(response.status as f64));
        resp_map.insert("statusText".to_string(), Value::Str(response.status_text));
        resp_map.insert("ok".to_string(), Value::Bool(response.ok));
        
        let mut headers_map = HashMap::new();
        for (k, v) in &response.headers {
            headers_map.insert(k.clone(), Value::Str(v.clone()));
        }
        resp_map.insert("headers".to_string(), Value::Object(headers_map));
        resp_map.insert("_type".to_string(), Value::Str("fetch_response".into()));
        resp_map.insert("_body_raw".to_string(), Value::Str(response.body));
        
        let resp_value = Value::Object(resp_map);

        // Create a Promise that's already fulfilled with the response
        let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
            crate::values::PromiseState::new()
        ));
        promise_state.borrow_mut().status = crate::values::PromiseStatus::Fulfilled;
        promise_state.borrow_mut().value = Some(resp_value);

        Ok(Value::Promise(promise_state))
    }

    fn vm_call_callback(&mut self, callback: &Value, args: Vec<Value>) -> Result<Value, String> {
        match callback {
            Value::BytecodeClosure(bc) => {
                let fn_data = &self.functions[bc.fn_idx];
                let num_params = fn_data.param_count;
                let offset = fn_data.param_offset;

                // Calculate total param slots (including destructured fields)
                let mut total_param_slots = num_params;
                for (_, fields) in &fn_data.destructured_params {
                    total_param_slots = total_param_slots - 1 + fields.len();
                }

                let captured_base = offset + total_param_slots;
                let total_slots = (captured_base + bc.captured.len()).max(16);
                let mut locals = vec![None; total_slots];

                // Slot 0 = self for recursion
                if offset > 0 {
                    locals[0] = Some(Value::BytecodeClosure(BytecodeClosure {
                        fn_idx: bc.fn_idx,
                        captured: bc.captured.clone(),
                        nesting_depth: bc.nesting_depth,
                    }));
                }

                // Place arguments with destructuring/rest handling
                let mut arg_idx = 0;
                let mut param_slot = offset;

                for i in 0..num_params {
                    if let Some((_, fields)) = fn_data
                        .destructured_params
                        .iter()
                        .find(|(param_idx, _)| *param_idx == i)
                    {
                        if arg_idx < args.len() {
                            let obj = args[arg_idx].clone();
                            locals[param_slot] = Some(obj.clone());
                            if let Value::Object(map) = &obj {
                                for (j, field) in fields.iter().enumerate() {
                                    if let Some(val) = map.get(field) {
                                        locals[param_slot + j] = Some(val.clone());
                                    }
                                }
                            }
                            arg_idx += 1;
                        }
                        param_slot += fields.len();
                    } else if fn_data.rest_slot == Some(i) {
                        let remaining: Vec<Value> = args[arg_idx..].to_vec();
                        locals[param_slot] = Some(Value::Array(remaining));
                        arg_idx = args.len();
                        param_slot += 1;
                    } else {
                        if arg_idx < args.len() {
                            locals[param_slot] = Some(args[arg_idx].clone());
                            arg_idx += 1;
                        }
                        param_slot += 1;
                    }
                }

                // Place captured values after all params
                for (i, val) in bc.captured.iter().enumerate() {
                    let slot = captured_base + i;
                    if slot < locals.len() {
                        locals[slot] = Some(val.clone());
                    }
                }

                self.frames.push(CallFrame {
                    fn_idx: bc.fn_idx,
                    ip: 0,
                    locals,
                });

                // Execute until this frame returns
                loop {
                    if self.frames.len() == 1 {
                        // Only one frame left - we're at the top level
                        return self.execute();
                    }
                    let frame_count = self.frames.len();
                    self.execute_one()?;
                    if self.frames.len() < frame_count {
                        // Frame returned — check if it's our callback frame
                        if self.frames.last().unwrap().fn_idx != bc.fn_idx
                            || self.frames.is_empty()
                        {
                            break;
                        }
                    }
                }

                Ok(self.value_stack.pop().unwrap_or(Value::Number(0.0)))
            }
            _ => Err("Callback is not a function".into()),
        }
    }

    fn execute_one(&mut self) -> Result<(), String> {
        if self.frames.is_empty() {
            return Ok(());
        }

        let frame = self.frames.last_mut().unwrap();
        let fn_idx = frame.fn_idx;
        let ip = frame.ip;
        let fn_data = &self.functions[fn_idx];
        let chunk = &fn_data.chunk;

        if ip >= chunk.code.len() {
            let r = self.value_stack.pop().unwrap_or(Value::Number(0.0));
            self.frames.pop();
            if !self.frames.is_empty() {
                self.value_stack.push(r);
            }
            return Ok(());
        }

        let op = chunk.code[ip].clone();
        frame.ip += 1;

        match op {
            OpCode::PushConst(idx) => {
                self.value_stack.push(chunk.constants[idx as usize].clone());
            }
            OpCode::Dup => {
                let v = self.value_stack.last().unwrap().clone();
                self.value_stack.push(v);
            }
            OpCode::Pop => {
                self.value_stack.pop();
            }
            OpCode::LoadLocal(slot) => {
                let slot = slot as usize;
                let frame = self.frames.last().unwrap();
                match frame.locals.get(slot).and_then(|v| v.as_ref()).cloned() {
                    Some(v) => self.value_stack.push(v),
                    None => {
                        return Err(format!("Undefined local at slot {}", slot));
                    }
                }
            }
            OpCode::StoreLocal(slot) => {
                let val = self.value_stack.pop().unwrap();
                let slot = slot as usize;
                let frame = self.frames.last_mut().unwrap();
                if slot >= frame.locals.len() {
                    frame.locals.resize(slot + 1, None);
                }
                frame.locals[slot] = Some(val.clone());
                self.value_stack.push(val);
            }
            OpCode::LoadGlobal(idx) => {
                let name = &chunk.globals[idx as usize];
                match self.globals.get(name) {
                    Some(v) => self.value_stack.push(v.clone()),
                    None => return Err(format!("Undefined variable: {}", name)),
                }
            }
            OpCode::StoreGlobal(idx) => {
                let v = self.value_stack.pop().unwrap();
                let name = chunk.globals[idx as usize].clone();
                self.globals.insert(name, v);
            }
            OpCode::Add => self.bin_op(BinOp::Add)?,
            OpCode::Sub => self.bin_op(BinOp::Sub)?,
            OpCode::Mul => self.bin_op(BinOp::Mul)?,
            OpCode::Div => self.bin_op(BinOp::Div)?,
            OpCode::Mod => self.bin_op(BinOp::Mod)?,
            OpCode::Eq => {
                let b = self.value_stack.pop().unwrap();
                let a = self.value_stack.pop().unwrap();
                self.value_stack.push(Value::Bool(a == b));
            }
            OpCode::Ne => {
                let b = self.value_stack.pop().unwrap();
                let a = self.value_stack.pop().unwrap();
                self.value_stack.push(Value::Bool(a != b));
            }
            OpCode::Lt => self.bin_op(BinOp::Lt)?,
            OpCode::Gt => self.bin_op(BinOp::Gt)?,
            OpCode::Le => self.bin_op(BinOp::Le)?,
            OpCode::Ge => self.bin_op(BinOp::Ge)?,
            OpCode::Jump(t) => {
                self.frames.last_mut().unwrap().ip = t;
            }
            OpCode::JumpIfFalse(t) => {
                if crate::values::is_falsey(self.value_stack.last().unwrap()) {
                    self.frames.last_mut().unwrap().ip = t;
                }
            }
            OpCode::JumpIfTrue(t) => {
                if crate::values::is_truthy(self.value_stack.last().unwrap()) {
                    self.frames.last_mut().unwrap().ip = t;
                }
            }
            OpCode::JumpIfFalsePop(t) => {
                let v = self.value_stack.pop().unwrap();
                if crate::values::is_falsey(&v) {
                    self.frames.last_mut().unwrap().ip = t;
                }
            }
            OpCode::JumpIfTruePop(t) => {
                let v = self.value_stack.pop().unwrap();
                if crate::values::is_truthy(&v) {
                    self.frames.last_mut().unwrap().ip = t;
                }
            }
            OpCode::ToBool => {
                let v = self.value_stack.pop().unwrap();
                self.value_stack.push(Value::Bool(crate::values::is_truthy(&v)));
            }
            OpCode::MakeObject(n) => {
                let mut m = HashMap::new();
                for _ in 0..n {
                    let v = self.value_stack.pop().unwrap();
                    let k = self.value_stack.pop().unwrap();
                    if let Value::Str(s) = k {
                        m.insert(s, v);
                    }
                }
                self.value_stack.push(Value::Object(m));
            }
            OpCode::Index => {
                let idx = self.value_stack.pop().unwrap();
                let base = self.value_stack.pop().unwrap();
                self.value_stack.push(vm_index(&base, &idx)?);
            }
            OpCode::GetProperty(idx) => {
                let prop = &chunk.properties[idx as usize];
                let v = self.value_stack.pop().unwrap();
                self.value_stack.push(vm_get_property(&v, prop)?);
            }
            OpCode::Spread => {
                let v = self.value_stack.pop().unwrap();
                if let Value::Array(items) = v {
                    for item in items {
                        self.value_stack.push(item);
                    }
                } else {
                    return Err(format!("Cannot spread non-array: {:?}", v));
                }
            }
            OpCode::PushDepthMarker => {
                self.value_stack.push(Value::Number(f64::NEG_INFINITY));
            }
            OpCode::MakeArray => {
                let mut elements = Vec::new();
                while let Some(top) = self.value_stack.last() {
                    if let Value::Number(n) = top {
                        if n.is_infinite() && n.is_sign_negative() {
                            self.value_stack.pop();
                            break;
                        }
                    }
                    elements.push(self.value_stack.pop().unwrap());
                }
                elements.reverse();
                self.value_stack.push(Value::Array(elements));
            }
            OpCode::MakeClosure(fn_idx) => {
                let fn_idx = fn_idx as usize;
                let fn_data = &self.functions[fn_idx];
                let frame = self.frames.last().unwrap();
                let captured: Vec<Value> = fn_data
                    .captured_slots
                    .iter()
                    .filter_map(|(ps, _)| {
                        frame
                            .locals
                            .get(*ps)
                            .and_then(|v| v.as_ref())
                            .cloned()
                    })
                    .collect();
                self.value_stack.push(Value::BytecodeClosure(BytecodeClosure {
                    fn_idx,
                    captured,
                    nesting_depth: fn_data.nesting_depth,
                }));
            }
            OpCode::Call(arg_count) => {
                self.vm_call(arg_count as usize)?;
            }
            OpCode::CallSpread(_) => {
                let mut closure_pos = 0;
                for i in (0..self.value_stack.len()).rev() {
                    if matches!(self.value_stack[i], Value::BytecodeClosure(_)) {
                        closure_pos = i;
                        break;
                    }
                }
                let total_args = self.value_stack.len() - closure_pos - 1;
                self.vm_call(total_args)?;
            }
            OpCode::TailCall(arg_count) => {
                self.vm_call(arg_count as usize)?;
            }
            OpCode::TailCallSpread(_) => {
                let mut closure_pos = 0;
                for i in (0..self.value_stack.len()).rev() {
                    if matches!(self.value_stack[i], Value::BytecodeClosure(_)) {
                        closure_pos = i;
                        break;
                    }
                }
                let total_args = self.value_stack.len() - closure_pos - 1;
                self.vm_call(total_args)?;
            }
            OpCode::MakePromise => {
                let executor = self.value_stack.pop().unwrap();
                let promise_state = std::rc::Rc::new(std::cell::RefCell::new(
                    crate::values::PromiseState::new()
                ));
                let resolve = Value::Resolver(promise_state.clone());
                let reject = Value::Rejector(promise_state.clone());
                self.vm_call_callback(&executor, vec![resolve, reject])?;
                self.value_stack.push(Value::Promise(promise_state));
            }
            OpCode::MethodCall(arg_count) => {
                self.vm_method_call(arg_count)?;
            }
            OpCode::Return => {
                let r = self.value_stack.pop().unwrap();
                self.frames.pop();
                if !self.frames.is_empty() {
                    self.value_stack.push(r);
                }
            }
        }
        Ok(())
    }

    fn bin_op(&mut self, op: crate::BinOp) -> Result<(), String> {
        let b = self.value_stack.pop().unwrap();
        let a = self.value_stack.pop().unwrap();
        self.value_stack.push(vm_bin_op(&a, &op, &b)?);
        Ok(())
    }

    // JSON helpers (sync, no promises)
    fn json_parse(&self, s: &str) -> Result<Value, String> {
        let t = s.trim();
        if t.starts_with('{') && t.ends_with('}') {
            Ok(Value::Object(json_parse_pairs(&t[1..t.len() - 1])?))
        } else if t.starts_with('[') && t.ends_with(']') {
            Ok(Value::Array(json_parse_items(&t[1..t.len() - 1])?))
        } else if t.starts_with('"') && t.ends_with('"') {
            Ok(Value::Str(t[1..t.len() - 1].to_string()))
        } else if t == "true" {
            Ok(Value::Bool(true))
        } else if t == "false" {
            Ok(Value::Bool(false))
        } else if t == "null" {
            Ok(Value::Number(0.0))
        } else if let Ok(n) = t.parse::<f64>() {
            Ok(Value::Number(n))
        } else {
            Err(format!("Cannot parse JSON: {}", s))
        }
    }

    fn json_stringify(&self, val: &Value, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let inner_pad = "  ".repeat(indent + 1);
        match val {
            Value::Number(n) => {
                if *n == (*n as i64) as f64 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(true) => "true".into(),
            Value::Bool(false) => "false".into(),
            Value::Str(s) => format!("\"{}\"", s),
            Value::Array(arr) => {
                if arr.is_empty() {
                    return "[]".into();
                }
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| format!("{}{}", inner_pad, self.json_stringify(v, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), pad)
            }
            Value::Object(map) => {
                if map.is_empty() {
                    return "{}".into();
                }
                let mut e: Vec<_> = map.iter().collect();
                e.sort_by(|a, b| a.0.cmp(b.0));
                let items: Vec<String> = e
                    .iter()
                    .map(|(k, v)| {
                        format!("{}\"{}\": {}", inner_pad, k, self.json_stringify(v, indent + 1))
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), pad)
            }
            _ => "[object]".into(),
        }
    }
}

// ---- standalone helpers ----

fn format_value_for_join(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        Value::Number(n) => format!("{}", *n as i64),
        _ => format_value_fb(v),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x - y).abs() < f64::EPSILON,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() { return false; }
            x.iter().zip(y.iter()).all(|(a, b)| values_equal(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            if x.len() != y.len() { return false; }
            x.iter().all(|(k, v)| y.get(k).map(|yv| values_equal(v, yv)).unwrap_or(false))
        }
        _ => false,
    }
}

fn format_value_fb(v: &Value) -> String {
    match v {
        Value::Number(n) => format!("{}", *n as i64),
        Value::Bool(true) => "true".into(),
        Value::Bool(false) => "false".into(),
        Value::Str(s) => s.clone(),
        Value::Array(a) => {
            let p: Vec<String> = a.iter().map(format_value_fb).collect();
            format!("[{}]", p.join(", "))
        }
        Value::Object(_) => "[object]".into(),
        _ => "[value]".into(),
    }
}

fn json_parse_pairs(s: &str) -> Result<HashMap<String, Value>, String> {
    let mut map = HashMap::new();
    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;
    while pos < chars.len() {
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }
        if chars[pos] == '"' {
            let mut key = String::new();
            pos += 1;
            while pos < chars.len() && chars[pos] != '"' {
                key.push(chars[pos]);
                pos += 1;
            }
            pos += 1;
            while pos < chars.len() && chars[pos] != ':' {
                pos += 1;
            }
            pos += 1;
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }
            let (vs, end) = json_extract_value(&chars, pos)?;
            pos = end;
            map.insert(key, json_parse_value(&vs)?);
            while pos < chars.len() && (chars[pos] == ',' || chars[pos].is_whitespace()) {
                pos += 1;
            }
        } else {
            pos += 1;
        }
    }
    Ok(map)
}

fn json_parse_items(s: &str) -> Result<Vec<Value>, String> {
    let mut items = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;
    while pos < chars.len() {
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        if pos >= chars.len() {
            break;
        }
        let (vs, end) = json_extract_value(&chars, pos)?;
        pos = end;
        items.push(json_parse_value(&vs)?);
        while pos < chars.len() && (chars[pos] == ',' || chars[pos].is_whitespace()) {
            pos += 1;
        }
    }
    Ok(items)
}

fn json_parse_value(s: &str) -> Result<Value, String> {
    let t = s.trim();
    if t.starts_with('{') && t.ends_with('}') {
        Ok(Value::Object(json_parse_pairs(&t[1..t.len() - 1])?))
    } else if t.starts_with('[') && t.ends_with(']') {
        Ok(Value::Array(json_parse_items(&t[1..t.len() - 1])?))
    } else if t.starts_with('"') && t.ends_with('"') {
        Ok(Value::Str(t[1..t.len() - 1].to_string()))
    } else if t == "true" {
        Ok(Value::Bool(true))
    } else if t == "false" {
        Ok(Value::Bool(false))
    } else if t == "null" {
        Ok(Value::Number(0.0))
    } else if let Ok(n) = t.parse::<f64>() {
        Ok(Value::Number(n))
    } else {
        Err(format!("Cannot parse JSON value: {}", s))
    }
}

fn json_extract_value(chars: &[char], start: usize) -> Result<(String, usize), String> {
    if start >= chars.len() {
        return Err("Unexpected end".into());
    }
    if chars[start] == '"' {
        let mut end = start + 1;
        while end < chars.len() && chars[end] != '"' {
            end += 1;
        }
        end += 1;
        Ok((chars[start..end].iter().collect::<String>(), end))
    } else if chars[start] == '{' || chars[start] == '[' {
        let br = chars[start];
        let cl = if br == '{' { '}' } else { ']' };
        let mut d = 1;
        let mut end = start + 1;
        while end < chars.len() && d > 0 {
            if chars[end] == br {
                d += 1;
            }
            if chars[end] == cl {
                d -= 1;
            }
            end += 1;
        }
        Ok((chars[start..end].iter().collect::<String>(), end))
    } else {
        let mut end = start;
        while end < chars.len() && chars[end] != ',' && chars[end] != '}' && chars[end] != ']' {
            end += 1;
        }
        Ok((
            chars[start..end].iter().collect::<String>().trim().to_string(),
            end,
        ))
    }
}

fn vm_bin_op(a: &Value, op: &BinOp, b: &Value) -> Result<Value, String> {
    match op {
        BinOp::Add => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (Value::Str(x), Value::Number(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (Value::Number(x), Value::Str(y)) => Ok(Value::Str(format!("{}{}", x, y))),
            (Value::Array(a1), Value::Array(a2)) => {
                let mut c = a1.clone();
                c.extend(a2.clone());
                Ok(Value::Array(c))
            }
            _ => Err(format!("Cannot add {:?} and {:?}", a, b)),
        },
        BinOp::Sub => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
            _ => Err(format!("Cannot subtract {:?} and {:?}", a, b)),
        },
        BinOp::Mul => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),
            _ => Err(format!("Cannot multiply {:?} and {:?}", a, b)),
        },
        BinOp::Div => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x / y)),
            _ => Err(format!("Cannot divide {:?} and {:?}", a, b)),
        },
        BinOp::Mod => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x % y)),
            _ => Err(format!("Cannot mod {:?} and {:?}", a, b)),
        },
        BinOp::Lt => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x < y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x < y)),
            _ => Err(format!("Cannot compare {:?} < {:?}", a, b)),
        },
        BinOp::Gt => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x > y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x > y)),
            _ => Err(format!("Cannot compare {:?} > {:?}", a, b)),
        },
        BinOp::Le => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x <= y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x <= y)),
            _ => Err(format!("Cannot compare {:?} <= {:?}", a, b)),
        },
        BinOp::Ge => match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x >= y)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Bool(x >= y)),
            _ => Err(format!("Cannot compare {:?} >= {:?}", a, b)),
        },
        _ => Err(format!("Binary op {:?} not supported in VM", op)),
    }
}

fn vm_index(base: &Value, idx: &Value) -> Result<Value, String> {
    match (base, idx) {
        (Value::Array(arr), Value::Number(n)) => {
            let i = *n as usize;
            if i < arr.len() {
                Ok(arr[i].clone())
            } else {
                Err(format!("Index {} out of bounds", i))
            }
        }
        (Value::Str(s), Value::Number(n)) => {
            let i = *n as usize;
            let chars: Vec<char> = s.chars().collect();
            if i < chars.len() {
                Ok(Value::Str(chars[i].to_string()))
            } else {
                Ok(Value::Str(String::new()))
            }
        }
        (Value::Object(map), Value::Str(key)) => {
            map.get(key)
                .cloned()
                .ok_or_else(|| format!("Key '{}' not found", key))
        }
        (Value::Object(map), Value::Number(n)) => {
            let key = (*n as i64).to_string();
            map.get(&key)
                .cloned()
                .ok_or_else(|| format!("Key '{}' not found", key))
        }
        _ => Err(format!("Cannot index {:?} with {:?}", base, idx)),
    }
}

fn vm_get_property(val: &Value, prop: &str) -> Result<Value, String> {
    match val {
        Value::Object(map) => map
            .get(prop)
            .cloned()
            .ok_or_else(|| format!("Property '{}' not found", prop)),
        Value::Array(arr) => match prop {
            "length" => Ok(Value::Number(arr.len() as f64)),
            _ => Err(format!("Array has no property '{}'", prop)),
        },
        Value::Str(s) => match prop {
            "length" => Ok(Value::Number(s.len() as f64)),
            _ => Err(format!("String has no property '{}'", prop)),
        },
        Value::Expect { value, negate } => match prop {
            "not" => Ok(Value::Expect {
                value: value.clone(),
                negate: !negate,
            }),
            _ => Err(format!("Expect has no property '{}'", prop)),
        },
        _ => Err(format!("Cannot access property '{}' of {:?}", prop, val)),
    }
}
