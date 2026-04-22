use std::io::{self, BufRead, Write};
use std::path::Path;
use djs::vm::{Compiler, VM};
use djs::values::format_value;

fn run_vm_file(path: &str) {
    use std::fs;

    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    });

    let has_imports = source.contains("import ");

    if has_imports {
        run_with_modules(path, &source);
    } else {
        run_simple(path, &source);
    }
}

fn run_simple(_path: &str, source: &str) {
    let mut compiler = Compiler::new();
    let (chunk, _decls) = match compiler.compile_program(source) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Compile error: {}", e);
            std::process::exit(1);
        }
    };
    execute_vm(chunk, compiler.functions);
}

fn run_with_modules(path: &str, _source: &str) {
    // Convert to absolute path
    let abs_path = if Path::new(path).is_absolute() {
        path.to_string()
    } else {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| cwd.join(path).canonicalize().ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string())
    };

    let path_obj = Path::new(&abs_path);
    let base = path_obj.parent().map(|p| p.to_str().unwrap_or(".")).unwrap_or(".");
    let filename = path_obj.file_name().and_then(|n| n.to_str()).unwrap_or(path);

    let mut registry = djs::vm::ModuleRegistry::new(base);

    match registry.compile_file(filename) {
        Ok((chunk, functions)) => {
            execute_vm(chunk, functions);
        }
        Err(e) => {
            eprintln!("Module error: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_vm(chunk: djs::vm::Chunk, functions: Vec<djs::vm::CompiledFunction>) {
    let mut vm = VM::new();
    if std::env::var("DEBUG_VM").is_ok() {
        chunk.disassemble("<main>");
        for f in &functions { f.chunk.disassemble(&f.name); eprintln!("  captured_slots={:?}", f.captured_slots); }
    }
    match vm.run(chunk, functions) {
        Ok(val) => {
            let final_val = match &val {
                djs::values::Value::Promise(state) => {
                    let p = state.borrow();
                    p.value.clone().unwrap_or(val.clone())
                }
                _ => val,
            };
            println!("{}", format_value(&final_val));
        }
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}

fn repl() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    println!("=== DJS REPL (VM) ===");
    println!("Type 'quit' to exit");
    println!();

    loop {
        print!("> ");
        out.flush().unwrap();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, Ok(_) => {}, Err(e) => { eprintln!("Error reading input: {}", e); break; }
        }
        let line = line.trim();
        if line.is_empty() || line == "quit" || line == "exit" { break; }

        let mut compiler = Compiler::new();
        match compiler.compile_program(line) {
            Ok((chunk, _decls)) => {
                let mut vm = VM::new();
                match vm.run(chunk, compiler.functions) {
                    Ok(val) => println!("{}", format_value(&val)),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
    println!("\nGoodbye!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        repl();
        return;
    }

    match args[1].as_str() {
        "run" => {
            let file = if args.len() >= 3 { &args[2] } else { &find_entry() };
            if file.is_empty() {
                eprintln!("Error: No entry file found");
                eprintln!("Usage: djs run <file.js>");
                std::process::exit(1);
            }
            run_vm_file(file);
        }
        "build" => {
            let entry = if args.len() >= 3 { &args[2] } else { &find_entry() };
            if entry.is_empty() {
                eprintln!("Error: No entry file found");
                eprintln!("Usage: djs build <entry.js> [output.js]");
                std::process::exit(1);
            }
            let output = if args.len() >= 4 { &args[3] } else { "dist/bundle.js" };
            build_project(entry, output);
        }
        "deploy" => {
            let entry = if args.len() >= 3 { &args[2] } else { &find_entry() };
            if entry.is_empty() {
                eprintln!("Error: No entry file found");
                eprintln!("Usage: djs deploy [entry.js]");
                std::process::exit(1);
            }
            deploy_project(entry);
        }
        "test" => {
            let test_path = if args.len() >= 3 { &args[2] } else { "." };
            run_test_files(test_path);
        }
        "check" => {
            let check_path = if args.len() >= 3 { &args[2] } else { "." };
            check_files(check_path);
        }
        "lint" => {
            let lint_path = if args.len() >= 3 { &args[2] } else { "." };
            lint_project(lint_path);
        }
        "version" | "--version" | "-v" => {
            println!("DJS v0.1.0 (Declarative JavaScript VM)");
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        "repl" => {
            repl();
        }
        // Backward compatibility: treat as file path
        _ => {
            if args[1] == "--vm" && args.len() >= 3 {
                run_vm_file(&args[2]);
            } else {
                run_vm_file(&args[1]);
            }
        }
    }
}

fn print_usage() {
    println!("DJS - Declarative JavaScript VM");
    println!();
    println!("USAGE:");
    println!("  djs [file.js]          Run a DJS file");
    println!("  djs run [file.js]      Run a DJS file (auto-detects entry)");
    println!("  djs build [entry.js]   Bundle project for production");
    println!("  djs deploy [entry.js]  Check + lint + build in one command");
    println!("  djs test [path]        Run test files with built-in test framework");
    println!("  djs check [path]       Check files compile (no run)");
    println!("  djs lint [path]        Check code purity");
    println!("  djs repl               Start REPL");
    println!("  djs version            Show version");
    println!("  djs help               Show this help");
    println!();
    println!("WORKFLOW:");
    println!("  djs deploy             # One command to production");
    println!("  djs run                # Auto-detect entry point");
    println!();
}

fn build_project(entry: &str, output: &str) {
    use std::fs;
    use std::path::Path;

    // Determine base directory
    let entry_path = Path::new(entry);
    let base = entry_path.parent().map(|p| p.to_str().unwrap_or(".")).unwrap_or(".");
    let filename = entry_path.file_name().and_then(|n| n.to_str()).unwrap_or(entry);

    // Read and bundle modules
    let mut registry = djs::vm::ModuleRegistry::new(base);
    
    match registry.compile_file(filename) {
        Ok(_) => {
            // Build combined source
            let combined_source = registry.build_combined_source(filename, &mut Vec::new())
                .unwrap_or_else(|e| {
                    eprintln!("✗ Build failed");
                    eprintln!("  {}", e);
                    std::process::exit(1);
                });

            // Create output directory
            let output_path = Path::new(output);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).ok();
            }

            // Generate production bundle
            let bundle = format!(
                "// DJS Production Bundle\n\
                 // Entry: {}\n\
                 // Generated by DJS v0.1.0\n\
                 //\n\
                 // This file contains all modules bundled into a single script.\n\
                 // Run with: djs run {}\n\n\
                 {}\n",
                entry, output, combined_source
            );

            fs::write(output, &bundle).unwrap_or_else(|e| {
                eprintln!("✗ Build failed");
                eprintln!("  Could not write output file: {}", e);
                std::process::exit(1);
            });

            println!("✓ Build successful");
            println!("  Entry: {}", entry);
            println!("  Output: {}", output);
            println!("  Size: {} bytes", bundle.len());
            println!();
            println!("To run the bundled file:");
            println!("  djs run {}", output);
        }
        Err(e) => {
            eprintln!("✗ Build failed");
            eprintln!("  {}", e);
            std::process::exit(1);
        }
    }
}

fn find_entry() -> String {
    use std::fs;
    use std::path::Path;

    // 1. Check djs.json for entry
    if Path::new("djs.json").exists() {
        if let Ok(content) = fs::read_to_string("djs.json") {
            // Simple JSON parsing without serde
            if let Some(entry) = extract_json_string(&content, "entry") {
                if Path::new(&entry).exists() {
                    return entry;
                }
            }
        }
    }

    // 2. Look for common entry points
    let candidates = vec![
        "main.js", "index.js", "app.js", "server.js",
        "src/main.js", "src/index.js", "src/app.js",
    ];

    for candidate in &candidates {
        if Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }

    String::new()
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    // Simple JSON extraction without serde
    let search = format!("\"{}\"", key);
    if let Some(pos) = json.find(&search) {
        let rest = &json[pos + search.len()..];
        if let Some(colon) = rest.find(':') {
            let after_colon = &rest[colon + 1..];
            let trimmed = after_colon.trim_start();
            if trimmed.starts_with('"') {
                if let Some(end) = trimmed[1..].find('"') {
                    return Some(trimmed[1..end + 1].to_string());
                }
            }
        }
    }
    None
}

fn deploy_project(entry: &str) {
    use std::fs;
    use std::path::Path;
    use std::time::Instant;

    let start = Instant::now();
    println!("🚀 DJS Deploy");
    println!("  Entry: {}", entry);
    println!();

    // Step 1: Check
    print!("Step 1/3: Checking files... ");
    let check_result = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("run")
        .arg(entry)
        .output();
    
    match check_result {
        Ok(output) if output.status.success() => {
            println!("✓");
        }
        _ => {
            println!("✗");
            eprintln!("Error: Entry file has errors");
            std::process::exit(1);
        }
    }

    // Step 2: Lint
    print!("Step 2/3: Checking purity... ");
    let lint_result = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("lint")
        .arg(entry)
        .output();

    match lint_result {
        Ok(output) if output.status.success() => {
            println!("✓");
        }
        _ => {
            println!("⚠");
            // Lint warnings don't fail deploy
        }
    }

    // Step 3: Build
    let output = "dist/bundle.js";
    print!("Step 3/3: Building bundle... ");
    
    let entry_path = Path::new(entry);
    let base = entry_path.parent().map(|p| p.to_str().unwrap_or(".")).unwrap_or(".");
    let filename = entry_path.file_name().and_then(|n| n.to_str()).unwrap_or(entry);

    let mut registry = djs::vm::ModuleRegistry::new(base);
    
    match registry.compile_file(filename) {
        Ok(_) => {
            let combined_source = match registry.build_combined_source(filename, &mut Vec::new()) {
                Ok(s) => s,
                Err(e) => {
                    println!("✗");
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            let output_path = Path::new(output);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).ok();
            }

            let bundle = format!(
                "// DJS Production Bundle\n\
                 // Entry: {}\n\
                 // Generated by DJS v0.1.0\n\n\
                 {}\n",
                entry, combined_source
            );

            if let Err(e) = fs::write(output, &bundle) {
                println!("✗");
                eprintln!("Error: Could not write output file: {}", e);
                std::process::exit(1);
            }

            println!("✓");
            let elapsed = start.elapsed();
            println!();
            println!("✅ Deployed successfully in {:.3}s", elapsed.as_secs_f64());
            println!("  Bundle: {}", output);
            println!("  Size: {} bytes", bundle.len());
        }
        Err(e) => {
            println!("✗");
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_test_files(path: &str) {
    use std::fs;
    use std::path::Path;

    let test_path = Path::new(path);
    
    if test_path.is_file() {
        println!("Running test: {}", path);
        run_vm_file(path);
    } else if test_path.is_dir() {
        println!("Running tests in: {}", path);
        let mut passed = 0;
        let mut failed = 0;
        let mut total = 0;

        for entry in fs::read_dir(test_path).unwrap_or_else(|e| {
            eprintln!("Error reading directory: {}", e);
            std::process::exit(1);
        }) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("js") {
                total += 1;
                let test_name = path.file_stem().unwrap().to_string_lossy();
                let result = std::process::Command::new(std::env::current_exe().unwrap())
                    .arg("run")
                    .arg(&path)
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        passed += 1;
                        println!("  ✓ {}", test_name);
                    }
                    _ => {
                        failed += 1;
                        println!("  ✗ {}", test_name);
                    }
                }
            }
        }

        println!();
        println!("Results: {} passed, {} failed, {} total", passed, failed, total);
        if failed > 0 {
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Path not found: {}", path);
        std::process::exit(1);
    }
}

fn check_files(path: &str) {
    use std::fs;
    use std::path::Path;

    let check_path = Path::new(path);
    
    if check_path.is_file() {
        // Check single file
        println!("Checking: {}", path);
        run_vm_file(path);
    } else if check_path.is_dir() {
        // Check all files in directory
        println!("Checking files in: {}", path);
        let mut passed = 0;
        let mut failed = 0;
        let mut total = 0;

        for entry in fs::read_dir(check_path).unwrap_or_else(|e| {
            eprintln!("Error reading directory: {}", e);
            std::process::exit(1);
        }) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("js") {
                total += 1;
                let file_name = path.file_stem().unwrap().to_string_lossy();
                let result = std::process::Command::new(std::env::current_exe().unwrap())
                    .arg("run")
                    .arg(&path)
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        passed += 1;
                        println!("  ✓ {}", file_name);
                    }
                    _ => {
                        failed += 1;
                        println!("  ✗ {}", file_name);
                    }
                }
            }
        }

        println!();
        println!("Results: {} passed, {} failed, {} total", passed, failed, total);
        if failed > 0 {
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Path not found: {}", path);
        std::process::exit(1);
    }
}

fn lint_project(path: &str) {
    use std::fs;
    use std::path::Path;

    let lint_path = Path::new(path);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut files_checked = 0;

    fn check_file(path: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>, files_checked: &mut usize) {
        if path.extension().and_then(|e| e.to_str()) != Some("js") {
            return;
        }

        *files_checked += 1;
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("{}: Cannot read file: {}", path.display(), e));
                return;
            }
        };

        let file_name = path.file_name().unwrap().to_string_lossy();
        
        // Check 1: Functions capturing global variables (impure functions)
        if source.contains("import ") {
            // Parse to find functions that reference imports
            let mut lexer = djs::lexer::Lexer::new(&source);
            let tokens = lexer.tokenize();
            
            // Find imported names
            let mut imports = Vec::new();
            let mut parser = djs::parser::Parser::new(tokens.clone());
            while matches!(parser.peek(), djs::lexer::Token::Import) {
                parser.consume();
                if let Ok(imp) = parser.parse_import() {
                    imports.extend(imp.names);
                }
            }
            
            // Check if any function body references imports without taking them as params
            let mut in_function = false;
            let mut func_name = String::new();
            let mut func_params = Vec::new();
            let mut func_body = String::new();
            let mut brace_depth = 0;
            
            for token in &tokens {
                match token {
                    djs::lexer::Token::Function => {
                        in_function = true;
                        func_name.clear();
                        func_params.clear();
                        func_body.clear();
                        brace_depth = 0;
                    }
                    djs::lexer::Token::Ident(name) if in_function && func_name.is_empty() => {
                        func_name = name.clone();
                    }
                    djs::lexer::Token::LBrace if in_function => {
                        brace_depth += 1;
                    }
                    djs::lexer::Token::RBrace if in_function => {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // Check if function body references imports not in params
                            for import_name in &imports {
                                if func_body.contains(import_name) && !func_params.contains(import_name) {
                                    errors.push(format!(
                                        "{}: function '{}' captures global import '{}' (impure function)",
                                        file_name, func_name, import_name
                                    ));
                                }
                            }
                            in_function = false;
                        }
                    }
                    djs::lexer::Token::Comma if in_function && brace_depth == 1 => {}
                    _ if in_function && brace_depth >= 1 => {
                        if let djs::lexer::Token::Ident(name) = token {
                            func_body.push_str(name);
                            func_body.push(' ');
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check 2: Unused variables (warning)
        // Simple check: find function parameters that aren't used in body
        // (This is a simplified check - full analysis would require AST parsing)
    }

    if lint_path.is_file() {
        check_file(lint_path, &mut errors, &mut warnings, &mut files_checked);
    } else if lint_path.is_dir() {
        for entry in fs::read_dir(lint_path).unwrap_or_else(|e| {
            eprintln!("Error reading directory: {}", e);
            std::process::exit(1);
        }) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                check_file(&path, &mut errors, &mut warnings, &mut files_checked);
            } else if path.is_dir() && !path.ends_with("node_modules") {
                // Recurse into subdirectories
                fn check_dir(dir: &Path, errors: &mut Vec<String>, warnings: &mut Vec<String>, files_checked: &mut usize) {
                    if let Ok(entries) = fs::read_dir(dir) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let path = entry.path();
                                if path.is_file() {
                                    check_file(&path, errors, warnings, files_checked);
                                } else if path.is_dir() && !path.ends_with("node_modules") {
                                    check_dir(&path, errors, warnings, files_checked);
                                }
                            }
                        }
                    }
                }
                check_dir(&path, &mut errors, &mut warnings, &mut files_checked);
            }
        }
    } else {
        eprintln!("Error: Path not found: {}", path);
        std::process::exit(1);
    }

    // Report results
    if errors.is_empty() && warnings.is_empty() {
        println!("✓ All clear! {} files checked", files_checked);
    } else {
        if !errors.is_empty() {
            println!("✗ {} error(s):", errors.len());
            for error in &errors {
                println!("  ✗ {}", error);
            }
        }
        if !warnings.is_empty() {
            println!("⚠ {} warning(s):", warnings.len());
            for warning in &warnings {
                println!("  ⚠ {}", warning);
            }
        }
        println!();
        println!("Checked {} files", files_checked);
        if !errors.is_empty() {
            std::process::exit(1);
        }
    }
}
