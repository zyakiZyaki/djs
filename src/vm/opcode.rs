// Bytecode instruction set for the DJS VM
// Stack-based architecture

use std::fmt;
use crate::values::Value;

/// A single bytecode instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // === Stack ===
    PushConst(u32),
    Dup,
    Pop,

    // === Variables ===
    LoadLocal(u32),
    StoreLocal(u32),
    LoadGlobal(u32),
    StoreGlobal(u32),

    // === Arithmetic ===
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Gt, Le, Ge,

    // === Control Flow ===
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    JumpIfFalsePop(usize),
    JumpIfTruePop(usize),
    ToBool,

    // === Data Structures ===
    /// Push a special depth marker onto the stack
    PushDepthMarker,
    /// Pop all items after the depth marker, create array
    MakeArray,
    MakeObject(u16),
    Index,
    GetProperty(u32),
    Spread,

    // === Functions ===
    MakeClosure(u32),   // push closure for function index
    Call(u16),          // call function with N args
    CallSpread(u16),    // call function with spread args (N non-spread args)
    TailCall(u16),      // tail call optimization - reuse current frame
    TailCallSpread(u16),
    MakePromise,        // create Promise and call executor with resolve/reject
    MethodCall(u16),    // call method on object
    Return,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::PushConst(i) => write!(f, "PushConst({})", i),
            OpCode::Dup => write!(f, "Dup"),
            OpCode::Pop => write!(f, "Pop"),
            OpCode::LoadLocal(i) => write!(f, "LoadLocal({})", i),
            OpCode::StoreLocal(i) => write!(f, "StoreLocal({})", i),
            OpCode::LoadGlobal(i) => write!(f, "LoadGlobal({})", i),
            OpCode::StoreGlobal(i) => write!(f, "StoreGlobal({})", i),
            OpCode::Add => write!(f, "Add"),
            OpCode::Sub => write!(f, "Sub"),
            OpCode::Mul => write!(f, "Mul"),
            OpCode::Div => write!(f, "Div"),
            OpCode::Mod => write!(f, "Mod"),
            OpCode::Eq => write!(f, "Eq"),
            OpCode::Ne => write!(f, "Ne"),
            OpCode::Lt => write!(f, "Lt"),
            OpCode::Gt => write!(f, "Gt"),
            OpCode::Le => write!(f, "Le"),
            OpCode::Ge => write!(f, "Ge"),
            OpCode::Jump(t) => write!(f, "Jump({})", t),
            OpCode::JumpIfFalse(t) => write!(f, "JumpIfFalse({})", t),
            OpCode::JumpIfTrue(t) => write!(f, "JumpIfTrue({})", t),
            OpCode::JumpIfFalsePop(t) => write!(f, "JumpIfFalsePop({})", t),
            OpCode::JumpIfTruePop(t) => write!(f, "JumpIfTruePop({})", t),
            OpCode::ToBool => write!(f, "ToBool"),
            OpCode::PushDepthMarker => write!(f, "PushDepthMarker"),
            OpCode::MakeArray => write!(f, "MakeArray"),
            OpCode::MakeObject(n) => write!(f, "MakeObject({})", n),
            OpCode::Index => write!(f, "Index"),
            OpCode::GetProperty(i) => write!(f, "GetProperty({})", i),
            OpCode::Spread => write!(f, "Spread"),
            OpCode::MakeClosure(i) => write!(f, "MakeClosure({})", i),
            OpCode::Call(n) => write!(f, "Call({})", n),
            OpCode::CallSpread(n) => write!(f, "CallSpread({})", n),
            OpCode::TailCall(n) => write!(f, "TailCall({})", n),
            OpCode::TailCallSpread(n) => write!(f, "TailCallSpread({})", n),
            OpCode::MakePromise => write!(f, "MakePromise"),
            OpCode::MethodCall(n) => write!(f, "MethodCall({})", n),
            OpCode::Return => write!(f, "Return"),
        }
    }
}

/// A compiled chunk of bytecode.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub properties: Vec<String>,
    pub globals: Vec<String>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            properties: Vec::new(),
            globals: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, val: Value) -> u32 {
        let idx = self.constants.len() as u32;
        self.constants.push(val);
        idx
    }

    pub fn add_property(&mut self, name: &str) -> u32 {
        if let Some(idx) = self.properties.iter().position(|p| p == name) {
            return idx as u32;
        }
        let idx = self.properties.len() as u32;
        self.properties.push(name.to_string());
        idx
    }

    pub fn add_global(&mut self, name: &str) -> u32 {
        if let Some(idx) = self.globals.iter().position(|g| g == name) {
            return idx as u32;
        }
        let idx = self.globals.len() as u32;
        self.globals.push(name.to_string());
        idx
    }

    pub fn emit(&mut self, op: OpCode) {
        self.code.push(op);
    }

    pub fn patch_jump(&mut self, offset: usize, target: usize) {
        if let Some(op) = self.code.get_mut(offset) {
            match op {
                OpCode::JumpIfFalse(t) => *t = target,
                OpCode::JumpIfTrue(t) => *t = target,
                OpCode::Jump(t) => *t = target,
                OpCode::JumpIfFalsePop(t) => *t = target,
                OpCode::JumpIfTruePop(t) => *t = target,
                OpCode::ToBool => {} // No-op for patch
                other => panic!("Cannot patch non-jump instruction: {:?}", other),
            }
        }
    }

    pub fn code_offset(&self) -> usize {
        self.code.len()
    }

    pub fn disassemble(&self, name: &str) {
        println!("\n== {} ==", name);
        for (i, op) in self.code.iter().enumerate() {
            println!("  {:>4}: {}", i, op);
        }
    }
}

/// A compiled bytecode function.
#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub chunk: Chunk,
    pub param_count: usize,
    pub name: String,
    pub captured_slots: Vec<(usize, usize)>,
    pub rest_slot: Option<usize>,
    pub destructured_params: Vec<(usize, Vec<String>)>,
    pub param_offset: usize,
    pub nesting_depth: usize, // 0 = top-level, 1+ = nested inside N functions
}
