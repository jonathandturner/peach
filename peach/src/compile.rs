use bytecode::{builtin_type, Bytecode, BytecodeEngine, Definition, DefinitionId, Fun};
use time::PreciseTime;

struct CFile {
    output_src: String,
    expression_stack: Vec<String>,
}

impl CFile {
    fn new() -> CFile {
        CFile {
            output_src: String::new(),
            expression_stack: vec![],
        }
    }

    fn codegen_raw(&mut self, output: &str) {
        self.output_src += output;
    }

    fn delay_expr(&mut self, expr: String) {
        self.expression_stack.push(expr);
    }

    fn codegen_stmt(&mut self, stmt: &str) {
        for expr in self.expression_stack.drain(0..) {
            self.output_src += &expr;
            self.output_src += ";\n";
        }

        self.output_src += stmt;
    }
}

fn codegen_type(bc: &BytecodeEngine, definition_id: DefinitionId) -> String {
    let codegen_ty = match definition_id {
        builtin_type::U64 => "unsigned long long".into(),
        builtin_type::U32 => "unsigned".into(),
        builtin_type::I64 => "signed long long".into(),
        builtin_type::I32 => "signed".into(),
        builtin_type::UNKNOWN_INT => "int".into(),
        builtin_type::VOID_PTR => "void*".into(),
        builtin_type::VOID => "void".into(),
        builtin_type::BOOL => "bool".into(),
        ty => {
            if let Definition::Struct(_) = bc.definitions[definition_id] {
                //For now, let's assume all custom types are structs
                format!("struct struct_{}", ty)
            } else {
                unimplemented!("Expected struct during codegen_type");
            }
        }
    };
    codegen_ty
}

fn codegen_fn(cfile: &mut CFile, bc: &BytecodeEngine, fn_name: &str, fun: &Fun) {
    let mut next_temp_id = 0;

    //TODO: This isn't the best solution, but it's an experiment
    let mut temp_id_stack = vec![];

    cfile.codegen_raw(&format!(
        "{} {}(",
        &codegen_type(bc, fun.return_ty),
        fn_name
    ));

    let mut first = true;
    for param in &fun.params {
        cfile.codegen_raw(&format!(
            "{}{} {}",
            if !first { ", " } else { "" },
            codegen_type(bc, param.ty),
            param.name
        ));
        first = false;
    }

    cfile.codegen_raw(") {\n");

    for param in &fun.params {
        cfile.codegen_raw(&format!(
            "{} v{} = {};\n",
            codegen_type(bc, param.ty),
            param.var_id,
            param.name
        ));
    }

    for code in &fun.bytecode {
        match code {
            Bytecode::ReturnVoid => {
                cfile.codegen_stmt("return;\n");
                break;
            }
            Bytecode::ReturnLastStackValue => {
                let retval = cfile.expression_stack.pop().unwrap();
                cfile.codegen_stmt(&format!("return {};\n", retval));
                break;
            }
            Bytecode::As(ty) => {
                let val = cfile.expression_stack.pop().unwrap();
                cfile.delay_expr(format!("(({})({}))", codegen_type(bc, *ty), val));
            }
            Bytecode::PushU64(val) => {
                cfile.delay_expr(val.to_string());
            }
            Bytecode::PushU32(val) => {
                cfile.delay_expr(val.to_string());
            }
            Bytecode::PushI64(val) => {
                cfile.delay_expr(val.to_string());
            }
            Bytecode::PushI32(val) => {
                cfile.delay_expr(val.to_string());
            }
            Bytecode::PushUnknownInt(val) => {
                cfile.delay_expr(val.to_string());
            }
            Bytecode::PushRawPtr(p) => {
                if p.is_null() {
                    cfile.delay_expr("NULL".to_string());
                } else {
                    unimplemented!("Unsupported pointer type");
                }
            }
            Bytecode::PushBool(val) => {
                cfile.delay_expr(val.to_string());
            }
            Bytecode::Neg => {
                let val = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("(-{})", val));
            }
            Bytecode::Add => {
                let rhs = cfile.expression_stack.pop().unwrap();
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("({}+{})", lhs, rhs));
                // For checked add
                //cfile.delay_expr(format!("add32({},{})", lhs, rhs));
            }
            Bytecode::Sub => {
                let rhs = cfile.expression_stack.pop().unwrap();
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("({}-{})", lhs, rhs));
            }
            Bytecode::Mul => {
                let rhs = cfile.expression_stack.pop().unwrap();
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("({}*{})", lhs, rhs));
            }
            Bytecode::Div => {
                let rhs = cfile.expression_stack.pop().unwrap();
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("({}/{})", lhs, rhs));
            }
            Bytecode::Lt => {
                let rhs = cfile.expression_stack.pop().unwrap();
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("({} < {})", lhs, rhs));
            }
            Bytecode::Dot(field) => {
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("{}.{}", lhs, field));
            }
            Bytecode::LValueDot(field) => {
                let lhs = cfile.expression_stack.pop().unwrap();

                cfile.delay_expr(format!("{}.{}", lhs, field));
            }
            Bytecode::VarDecl(var_id) => {
                let rhs = cfile.expression_stack.pop().unwrap();

                let var = &fun.vars[*var_id];

                cfile.codegen_stmt(&format!(
                    "{} v{} = {};\n",
                    codegen_type(bc, var.ty),
                    *var_id,
                    rhs
                ));
            }
            Bytecode::VarDeclUninit(var_id) => {
                let var = &fun.vars[*var_id];
                cfile.codegen_stmt(&format!("{} v{};\n", codegen_type(bc, var.ty), *var_id));
            }
            Bytecode::Var(var_id) => {
                cfile.delay_expr(format!("v{}", var_id));
            }
            Bytecode::LValueVar(var_id) => {
                cfile.delay_expr(format!("v{}", var_id));
            }
            Bytecode::Assign => {
                let lhs = cfile.expression_stack.pop().unwrap();
                let rhs = cfile.expression_stack.pop().unwrap();

                cfile.codegen_stmt(&format!("{} = {};\n", lhs, rhs));
            }
            Bytecode::Call(definition_id) => {
                if let Definition::Fun(ref fun) = bc.definitions[*definition_id] {
                    let mut expr_string = String::new();

                    match fun.extern_name {
                        Some(ref ex_name) => {
                            expr_string += &format!("{}(", ex_name);
                        }
                        None => {
                            expr_string += &format!("fun_{}(", definition_id);
                        }
                    }
                    let expression_stack_len = cfile.expression_stack.len();
                    let mut offset = fun.params.len();
                    while offset > 0 {
                        expr_string += &cfile.expression_stack[expression_stack_len - offset];
                        if offset > 1 {
                            expr_string += ", "
                        }
                        offset -= 1;
                    }

                    expr_string += ")";
                    for _ in 0..fun.params.len() {
                        cfile.expression_stack.pop();
                    }
                    cfile.delay_expr(expr_string);
                } else if let Definition::Struct(ref st) = bc.definitions[*definition_id] {
                    let mut expr_string = format!("init_struct_{}(", definition_id);
                    let expression_stack_len = cfile.expression_stack.len();

                    let mut offset = st.fields.len();
                    while offset > 0 {
                        expr_string += &cfile.expression_stack[expression_stack_len - offset];
                        if offset > 1 {
                            expr_string += ", "
                        }
                        offset -= 1;
                    }

                    expr_string += ")";
                    for _ in 0..st.fields.len() {
                        cfile.expression_stack.pop();
                    }

                    cfile.delay_expr(expr_string);
                } else {
                    unimplemented!("Attempt to call unprocessed function");
                }
            }
            Bytecode::If(_, ty) => {
                let cond = cfile.expression_stack.pop().unwrap();

                match *ty {
                    builtin_type::U64
                    | builtin_type::BOOL
                    | builtin_type::U32
                    | builtin_type::I64
                    | builtin_type::I32
                    | builtin_type::UNKNOWN_INT => {
                        cfile.codegen_stmt(&format!(
                            "{} t{};\n",
                            codegen_type(bc, *ty),
                            next_temp_id
                        ));
                        temp_id_stack.push(next_temp_id);
                        next_temp_id += 1;
                    }
                    _ => {}
                }

                cfile.codegen_stmt(&format!("if ({}) {{\n", cond));
            }
            Bytecode::Else(_, ty) => {
                if *ty != builtin_type::VOID {
                    let result = cfile.expression_stack.pop().unwrap();
                    cfile.codegen_stmt(&format!(
                        "t{} = {};\n",
                        temp_id_stack.last().unwrap(),
                        result
                    ));
                }
                cfile.codegen_stmt("} else {\n");
            }
            Bytecode::EndIf(ty) => {
                if *ty != builtin_type::VOID {
                    let result = cfile.expression_stack.pop().unwrap();
                    let temp_id = temp_id_stack.pop().unwrap();
                    cfile.codegen_stmt(&format!("t{} = {};\n}}\n", temp_id, result));
                    cfile.expression_stack.push(format!("t{}", temp_id));
                } else {
                    cfile.codegen_stmt("}\n");
                }
            }
            Bytecode::BeginWhile => {
                cfile.codegen_stmt("while(1) {\n");
            }
            Bytecode::WhileCond(_) => {
                let cond = cfile.expression_stack.pop().unwrap();
                cfile.codegen_stmt(&format!("if (!({})) break;\n", cond));
            }
            Bytecode::EndWhile(_) => {
                cfile.codegen_stmt("}\n");
            }
            Bytecode::DebugPrint(ty) => {
                let val = cfile.expression_stack.pop().unwrap();
                let result = match *ty {
                    builtin_type::VOID => "DEBUG: <void>".into(),
                    builtin_type::UNKNOWN => "DEBUG: <unknown>".into(),
                    builtin_type::BOOL
                    | builtin_type::U32
                    | builtin_type::U64
                    | builtin_type::I32
                    | builtin_type::I64
                    | builtin_type::UNKNOWN_INT => {
                        format!("printf(\"DEBUG: %d\\n\", ({}));\n", val)
                    }
                    _ => format!("printf(\"DEBUG: <custom type:%u>\\n\", ({}));\n", ty),
                };
                cfile.codegen_stmt(&result);
            }
        }
    }

    cfile.codegen_stmt("};\n");
}

fn codegen_c_from_bytecode(bc: &BytecodeEngine) -> String {
    let mut cfile = CFile::new();

    cfile.codegen_raw("#include <stdio.h>\n");
    cfile.codegen_raw("#include <stdbool.h>\n");
    /* 
    // If we wanted checked add/sub we can use something like this:
    cfile.codegen_raw("#include <assert.h>\n");

    cfile.codegen_raw("int add32(int lhs, int rhs) {\n");
    cfile.codegen_raw("int answer = lhs + rhs;\n");
    cfile.codegen_raw("assert((lhs & 0x8000) == (answer & 0x8000));\n");
    cfile.codegen_raw("return answer;\n");
    cfile.codegen_raw("}\n");

    cfile.codegen_raw("int sub32(int lhs, int rhs) {\n");
    cfile.codegen_raw("int answer = lhs - rhs;\n");
    cfile.codegen_raw("assert((lhs & 0x8000) == (answer & 0x8000));\n");
    cfile.codegen_raw("return answer;\n");
    cfile.codegen_raw("}\n");
    */

    let starting_fn_id = bc.scopes[0].definitions["main"];

    //TODO: FIXME: just make two separate strings and concat them rather than iterating twice
    for definition_id in 0..bc.definitions.len() {
        if let Definition::Fun(ref fun) = bc.definitions[definition_id] {
            if definition_id != starting_fn_id {
                match fun.extern_name {
                    Some(ref ex_name) => {
                        cfile.codegen_raw(&format!(
                            "{} {}(",
                            &codegen_type(bc, fun.return_ty),
                            ex_name
                        ));
                    }
                    None => {
                        let header = format!(
                            "{} fun_{}(\n",
                            codegen_type(bc, fun.return_ty),
                            definition_id
                        );
                        cfile.codegen_raw(&header);
                    }
                }

                let mut first = true;
                for param in &fun.params {
                    cfile.codegen_raw(&format!(
                        "{}{} {}",
                        if !first { ", " } else { "" },
                        codegen_type(bc, param.ty),
                        param.name
                    ));
                    first = false;
                }

                cfile.codegen_raw(");\n");
            }
        } else if let Definition::Struct(_) = bc.definitions[definition_id] {
            let struct_line = format!("struct struct_{};\n", definition_id);
            cfile.codegen_raw(&struct_line);
            let struct_init_line = format!(
                "{} init_struct_{}();\n",
                codegen_type(bc, definition_id),
                definition_id
            );
            cfile.codegen_raw(&struct_init_line);
        }
    }

    for definition_id in 0..bc.definitions.len() {
        // Only codegen definitions that we know, others may be types (and not have an associated codegen)
        if let Definition::Fun(ref fun) = bc.definitions[definition_id] {
            if fun.extern_name.is_none() {
                if definition_id == starting_fn_id {
                    codegen_fn(&mut cfile, bc, "main", fun);
                } else {
                    codegen_fn(&mut cfile, bc, &format!("fun_{}", definition_id), fun);
                }
            }
        } else if let Definition::Struct(ref st) = bc.definitions[definition_id] {
            if st.fields.len() == 0 {
                cfile.codegen_raw(&format!(
                    "struct struct_{} {{int dummy;}};\n",
                    definition_id
                ));
            } else {
                cfile.codegen_raw(&format!("struct struct_{} {{", definition_id));
                for field in &st.fields {
                    cfile.codegen_raw(&format!("{} {};\n", codegen_type(bc, field.1), field.0));
                }
                cfile.codegen_raw("};\n");
            };

            cfile.codegen_raw(&format!(
                "{} init_struct_{}(",
                codegen_type(bc, definition_id),
                definition_id
            ));

            let mut first = true;

            for field in &st.fields {
                cfile.codegen_raw(&format!(
                    "{}{} {}",
                    if !first { ", " } else { "" },
                    codegen_type(bc, field.1),
                    field.0
                ));
                first = false;
            }

            cfile.codegen_raw(") {\n");

            cfile.codegen_raw(&format!("{} temp = ", codegen_type(bc, definition_id)));
            cfile.codegen_raw("{");
            let mut first = true;
            if st.fields.len() > 0 {
                for field in &st.fields {
                    cfile.codegen_raw(&format!("{}{}", if !first { ", " } else { "" }, field.0,));
                    first = false;
                }
            } else {
                cfile.codegen_raw("0");
            }
            cfile.codegen_raw("};\n");
            cfile.codegen_raw("return temp;\n");
            cfile.codegen_raw("}\n");
        }
    }

    cfile.output_src
}

/// Compiles the project's bytecode to a give name.  
/// Returns the location of the compiled binary.
pub fn compile_bytecode(bc: &BytecodeEngine, output_fname: &str) -> ::std::io::Result<String> {
    let output = codegen_c_from_bytecode(bc);

    let path = {
        use std::fs::File;
        use std::io::prelude::*;
        use std::path::Path;

        let output_path = Path::new(output_fname);

        let dir = ::std::env::temp_dir();
        let path = Path::new(&dir)
            .join(output_path.file_name().unwrap())
            .with_extension("c");
        let mut file =
            File::create(path.clone()).expect(&format!("Can not create {:?} for output", path));
        file.write_all(&output.as_bytes())
            .expect("Failed to write output to .c file");
        path
    };

    compile_file(path)
}

#[cfg(windows)]
fn compile_file(path: ::std::path::PathBuf) -> ::std::io::Result<String> {
    let start = PreciseTime::now();
    use std::process::Command;

    let output_fname = String::new() + path.with_extension("exe").to_str().unwrap();
    let output_objname = String::new() + path.with_extension("obj").to_str().unwrap();

    let output = Command::new(r"cl.exe")
        .arg("/w")
        .arg(&format!("/Fe{}", output_fname))
        .arg(&format!("/Fo{}", output_objname))
        .arg(path)
        .output()?;
    let end = PreciseTime::now();
    let duration = start
        .to(end)
        .to_std()
        .expect("Can't convert duration to std duration");

    println!(
        "status: {} in {:.3} sec",
        if output.status.success() {
            "success"
        } else {
            "fail"
        },
        duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
    );

    if output.status.success() {
        Ok(output_fname)
    } else {
        use std::io::{Error, ErrorKind};

        let compile_stdout = String::from_utf8(output.stdout).unwrap();
        let compile_stderr = String::from_utf8(output.stderr).unwrap();

        let combined_compile_msg = compile_stdout + &compile_stderr;

        Err(Error::new(ErrorKind::Other, combined_compile_msg))
    }
}

#[cfg(unix)]
fn compile_file(path: ::std::path::PathBuf) -> ::std::io::Result<String> {
    let start = PreciseTime::now();
    use std::process::Command;
    let output_fname = String::new() + path.with_extension("").to_str().unwrap();

    let output = Command::new(r"clang")
        .arg("-w")
        .arg(path)
        .arg("-o")
        .arg(&output_fname)
        .output()
        .expect("failed to execute compiler");
    let end = PreciseTime::now();
    let duration = start
        .to(end)
        .to_std()
        .expect("Can't convert duration to std duration");

    println!(
        "status: {} in {:.3} sec",
        if output.status.success() {
            "success"
        } else {
            "fail"
        },
        duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
    );

    if output.status.success() {
        Ok(output_fname)
    } else {
        use std::io::{Error, ErrorKind};

        let compile_stdout = String::from_utf8(output.stdout).unwrap();
        let compile_stderr = String::from_utf8(output.stderr).unwrap();

        let combined_compile_msg = compile_stdout + &compile_stderr;

        Err(Error::new(ErrorKind::Other, combined_compile_msg))
    }
}
