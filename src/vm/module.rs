// Module system — resolve and load modules

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::rc::Rc;
use crate::lexer::Lexer;
use crate::parser::{Parser, FuncDecl, ImportStmt};
use crate::vm::compiler::Compiler;
use crate::vm::opcode::Chunk;

#[derive(Clone)]
pub struct Module {
    pub path: String,
    pub exports: HashMap<String, FuncDecl>,
    pub source: String,  // Original source (imports stripped)
}

pub struct ModuleRegistry {
    pub modules: HashMap<String, Module>,
    pub base_dir: PathBuf,
}

impl ModuleRegistry {
    pub fn new(base_dir: &str) -> Self {
        ModuleRegistry {
            modules: HashMap::new(),
            base_dir: PathBuf::from(base_dir),
        }
    }

    pub(crate) fn resolve_path(&self, path: &str) -> PathBuf {
        let p = PathBuf::from(path);
        if p.is_absolute() { return p; }
        let path_str = if p.extension().is_none() && !path.ends_with(".js") {
            format!("{}.js", path)
        } else {
            path.to_string()
        };
        let resolved = self.base_dir.join(&path_str);
        if resolved.exists() { return resolved; }
        let resolved2 = self.base_dir.join(&p);
        if resolved2.exists() { return resolved2; }
        PathBuf::from(path)
    }

    /// Parse source and return (imports, exports, source_without_imports)
    pub(crate) fn parse_source(&self, source: &str) -> Result<(Vec<ImportStmt>, HashMap<String, FuncDecl>, String), String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        let mut exports = HashMap::new();
        let mut imports = Vec::new();
        let mut code_parts = Vec::new();

        while !matches!(parser.peek(), crate::lexer::Token::EOF) {
            match parser.peek() {
                crate::lexer::Token::Import => {
                    parser.consume();
                    imports.push(parser.parse_import()?);
                }
                crate::lexer::Token::Export => {
                    parser.consume();
                    let exp = parser.parse_export()?;
                    exports.insert(exp.func.name.clone(), exp.func.clone());
                    code_parts.push(format!("function {}(/* body */)", exp.func.name));
                }
                crate::lexer::Token::Function => {
                    let decl = parser.parse_func_decl()?;
                    code_parts.push(format!("function {}(/* body */)", decl.name));
                    exports.entry(decl.name.clone()).or_insert(decl);
                }
                _ => {
                    // Trailing expression or other — skip
                    parser.parse_expr().ok();
                }
            }
        }

        let code = code_parts.join("\n");
        Ok((imports, exports, code))
    }

    pub fn load(&mut self, path: &str) -> Result<Rc<Module>, String> {
        let resolved = self.resolve_path(path);
        let canonical = resolved.to_string_lossy().to_string();

        if let Some(module) = self.modules.get(&canonical) {
            return Ok(Rc::new(module.clone()));
        }

        let source = fs::read_to_string(&resolved)
            .map_err(|e| format!("Cannot read module '{}': {}", path, e))?;

        let (imports, mut exports, _) = self.parse_source(&source)?;

        for imp in &imports {
            let dep = self.load(&imp.source)?;
            for name in &imp.names {
                if let Some(func) = dep.exports.get(name) {
                    exports.insert(name.clone(), func.clone());
                } else {
                    return Err(format!("Module '{}' does not export '{}'", imp.source, name));
                }
            }
        }

        let module = Module { path: canonical.clone(), exports, source };
        self.modules.insert(canonical, module.clone());
        Ok(Rc::new(module))
    }

    /// Build a combined source string: imported sources + main source
    /// Imports are resolved recursively and their FULL source is included.
    pub fn build_combined_source(&self, path: &str, visited: &mut Vec<String>) -> Result<String, String> {
        let resolved = self.resolve_path(path);
        let canonical = resolved.to_string_lossy().to_string();

        if visited.contains(&canonical) {
            return Ok(String::new()); // Already included, skip
        }
        visited.push(canonical.clone());

        let source = fs::read_to_string(&resolved)
            .map_err(|e| format!("Cannot read '{}': {}", path, e))?;

        // Parse to find imports
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        let mut import_sources = Vec::new();
        while !matches!(parser.peek(), crate::lexer::Token::EOF) {
            match parser.peek() {
                crate::lexer::Token::Import => {
                    parser.consume();
                    let imp = parser.parse_import()?;
                    import_sources.push(imp.source.clone());
                }
                _ => { parser.consume(); }
            }
        }

        // Recursively include imported sources
        let mut combined = String::new();
        for imp_source in &import_sources {
            let imp_code = self.build_combined_source(imp_source, visited)?;
            if !imp_code.is_empty() {
                combined.push_str(&imp_code);
                combined.push_str("\n\n");
            }
        }

        // Strip import statements from main source
        let mut lines = source.lines();
        let mut main_code = String::new();
        let mut in_import = false;
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                in_import = true;
            }
            if in_import && trimmed.ends_with(';') {
                in_import = false;
                continue;
            }
            if !in_import {
                main_code.push_str(line);
                main_code.push('\n');
            }
        }

        // Also strip "export " prefix from functions
        let main_code = main_code.replace("export function", "function");

        combined.push_str(&main_code);
        Ok(combined)
    }

    pub fn compile_file(&mut self, path: &str) -> Result<(Chunk, Vec<crate::vm::CompiledFunction>), String> {
        // Build combined source
        let combined_source = self.build_combined_source(path, &mut Vec::new())?;

        // Compile with imports as locals (not globals) for proper encapsulation
        let mut compiler = Compiler::new();
        let (chunk, _decls) = compiler.compile_program_with_mode(&combined_source, true)?;

        Ok((chunk, compiler.functions))
    }
}
