#[cfg(test)]
mod tests {
    use std::process::Command;

    //use super::*;
    use bytecode::BytecodeEngine;
    use compile;
    use eval;

    fn load_to_bc(fname: &str) -> BytecodeEngine {
        let mut bc = BytecodeEngine::new();

        // Step 1: Load up the parsed file so that we can lazily convert it
        bc.set_project_root("test_files");
        bc.load_file(fname);

        // Step 2: Convert to bytecode from the given location
        // We assume the starting function is found in scope 0, the starting scope
        bc.process_fn("main", 0);

        bc
    }

    fn run_test(fname: &str, eval_expect: &str, compile_expect: &str) {
        let bc = load_to_bc(fname);

        // Eval stage
        let mut eval_output = Some(String::new());
        eval::eval_engine(&bc, "main", &mut eval_output);
        assert_eq!(eval_expect, eval_output.unwrap().trim());

        // Compile stage
        let compile_result = compile::compile_bytecode(&bc, fname);
        assert!(compile_result.is_ok());

        let cmd = Command::new(compile_result.unwrap())
            .output()
            .expect("failed to execute test");

        let test_output = String::from_utf8(cmd.stdout).unwrap();
        assert_eq!(test_output.trim(), compile_expect);
    }

    fn run_bad_test(fname: &str, expected_error_msg: &str) {
        use std::panic;

        let result = panic::catch_unwind(|| {
            load_to_bc(fname);
        });

        match result {
            Err(e) => {
                let error_msg = e.downcast_ref::<String>().unwrap();
                assert!(error_msg.contains(expected_error_msg));
            }
            _ => panic!("Expected failing test is succeeding"),
        }
    }

    #[test]
    fn test_expr01() {
        run_test("expr01.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_expr02_add() {
        run_test("expr02_add.rs", "DEBUG: U64(5)", "DEBUG: 5");
    }

    #[test]
    fn test_expr02_sub() {
        run_test("expr02_sub.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_expr02_mul() {
        run_test("expr02_mul.rs", "DEBUG: U64(20)", "DEBUG: 20");
    }

    #[test]
    fn test_expr02_div() {
        run_test("expr02_div.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_expr03() {
        run_test("expr03.rs", "DEBUG: U64(10)", "DEBUG: 10");
    }

    #[test]
    fn test_expr04() {
        run_test("expr04.rs", "DEBUG: U64(10)", "DEBUG: 10");
    }

    #[test]
    fn test_expr05() {
        run_test("expr05.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_expr06() {
        run_test("expr06.rs", "DEBUG: U64(18)", "DEBUG: 18");
    }

    #[test]
    fn test_expr07() {
        run_test("expr07.rs", "DEBUG: Bool(true)", "DEBUG: 1");
    }

    #[test]
    fn test_expr08() {
        run_test("expr08.rs", "DEBUG: U64(12)", "DEBUG: 12");
    }

    #[test]
    fn test_expr09() {
        run_test("expr09.rs", "DEBUG: U64(14)", "DEBUG: 14");
    }

    #[test]
    fn test_expr10() {
        run_test("expr10.rs", "DEBUG: Bool(false)", "DEBUG: 0");
    }

    #[test]
    fn test_expr11() {
        run_test("expr11.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_expr12() {
        run_test("expr12.rs", "DEBUG: U64(16)", "DEBUG: 16");
    }

    #[test]
    fn test_expr13() {
        run_test("expr13.rs", "DEBUG: U64(28)", "DEBUG: 28");
    }

    #[test]
    fn test_expr14() {
        run_test("expr14.rs", "DEBUG: U32(13)", "DEBUG: 13");
    }

    #[test]
    fn test_expr_bad01() {
        run_bad_test("expr_bad01.rs", "Can't add values of");
    }

    #[test]
    fn test_fn01() {
        run_test("fn01.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_fn02() {
        run_test("fn02.rs", "DEBUG: U64(11)", "DEBUG: 11");
    }

    #[test]
    fn test_fn03() {
        run_test("fn03.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_fn04() {
        run_test("fn04.rs", "DEBUG: U64(5)", "DEBUG: 5");
    }

    #[test]
    fn test_fn05() {
        run_test("fn05.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_fn06() {
        run_test("fn06.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_fn07() {
        run_test("fn07.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_fn08() {
        run_test("fn08.rs", "DEBUG: U64(8)", "DEBUG: 8");
    }

    #[test]
    fn test_var01() {
        run_test("var01.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_var02() {
        run_test("var02.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_var03() {
        run_test("var03.rs", "DEBUG: Bool(true)", "DEBUG: 1");
    }

    #[test]
    fn test_var_bad01() {
        run_bad_test("var_bad01.rs", "used before being given a value");
    }

    #[test]
    fn test_infer01() {
        run_test("infer01.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_if01() {
        run_test("if01.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_if02() {
        run_test("if02.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_if03() {
        run_test("if03.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_if04() {
        run_test("if04.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_if05() {
        run_test("if05.rs", "DEBUG: U64(5)", "DEBUG: 5");
    }

    #[test]
    fn test_if06() {
        run_test("if06.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_mod01() {
        run_test("mod01.rs", "DEBUG: U64(1)", "DEBUG: 1");
    }

    #[test]
    fn test_mod02() {
        run_test("mod02.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_mod03() {
        run_test("mod03.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_mod04() {
        run_test("mod04.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_mod05() {
        run_test("mod05.rs", "DEBUG: U64(7)", "DEBUG: 7");
    }

    #[test]
    fn test_mod06() {
        run_test("mod06.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_mod07() {
        run_test("mod07.rs", "DEBUG: U64(1)", "DEBUG: 1");
    }

    #[test]
    fn test_mod08() {
        run_test("mod08.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_mod09() {
        run_test("mod09.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_mod10() {
        run_test("mod10.rs", "DEBUG: U64(23)", "DEBUG: 23");
    }

    #[test]
    fn test_mod_bad01() {
        run_bad_test("mod_bad01.rs", "not found in module");
    }

    #[test]
    fn test_scope01() {
        run_test("scope01.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_scope02() {
        run_test("scope02.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_scope_bad01() {
        run_bad_test("scope_bad01.rs", "Can not call function");
    }

    #[test]
    fn test_while01() {
        run_test("while01.rs", "DEBUG: U64(10)", "DEBUG: 10");
    }
}
