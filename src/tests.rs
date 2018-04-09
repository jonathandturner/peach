#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;

    //use super::*;
    use bytecode::BytecodeEngine;
    use compile;
    use eval;

    fn load_to_bc(fname: &str) -> BytecodeEngine {
        let mut file = File::open(fname).expect("Unable to open file");

        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");

        let mut bc = BytecodeEngine::new();

        // Step 1: Load up the parsed file so that we can lazily convert it
        bc.load_file(&src);

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
        run_test("test_files/expr01.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_expr02_add() {
        run_test("test_files/expr02_add.rs", "DEBUG: U64(5)", "DEBUG: 5");
    }

    #[test]
    fn test_expr02_sub() {
        run_test("test_files/expr02_sub.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_expr02_mul() {
        run_test("test_files/expr02_mul.rs", "DEBUG: U64(20)", "DEBUG: 20");
    }

    #[test]
    fn test_expr02_div() {
        run_test("test_files/expr02_div.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_expr03() {
        run_test("test_files/expr03.rs", "DEBUG: U64(10)", "DEBUG: 10");
    }

    #[test]
    fn test_expr04() {
        run_test("test_files/expr04.rs", "DEBUG: U64(10)", "DEBUG: 10");
    }

    #[test]
    fn test_expr05() {
        run_test("test_files/expr05.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_expr06() {
        run_test("test_files/expr06.rs", "DEBUG: U64(18)", "DEBUG: 18");
    }

    #[test]
    fn test_expr07() {
        run_test("test_files/expr07.rs", "DEBUG: Bool(true)", "DEBUG: 1");
    }

    #[test]
    fn test_expr08() {
        run_test("test_files/expr08.rs", "DEBUG: U64(12)", "DEBUG: 12");
    }

    #[test]
    fn test_expr09() {
        run_test("test_files/expr09.rs", "DEBUG: U64(14)", "DEBUG: 14");
    }

    #[test]
    fn test_expr10() {
        run_test("test_files/expr10.rs", "DEBUG: Bool(false)", "DEBUG: 0");
    }

    #[test]
    fn test_expr11() {
        run_test("test_files/expr11.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_expr12() {
        run_test("test_files/expr12.rs", "DEBUG: U64(16)", "DEBUG: 16");
    }

    #[test]
    fn test_expr13() {
        run_test("test_files/expr13.rs", "DEBUG: U64(28)", "DEBUG: 28");
    }

    #[test]
    fn test_expr14() {
        run_test("test_files/expr14.rs", "DEBUG: U32(13)", "DEBUG: 13");
    }

    #[test]
    fn test_expr_bad01() {
        run_bad_test("test_files/expr_bad01.rs", "Can't add values of");
    }

    #[test]
    fn test_fn01() {
        run_test("test_files/fn01.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_fn02() {
        run_test("test_files/fn02.rs", "DEBUG: U64(11)", "DEBUG: 11");
    }

    #[test]
    fn test_fn03() {
        run_test("test_files/fn03.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_fn04() {
        run_test("test_files/fn04.rs", "DEBUG: U64(5)", "DEBUG: 5");
    }

    #[test]
    fn test_fn05() {
        run_test("test_files/fn05.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_fn06() {
        run_test("test_files/fn06.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_fn07() {
        run_test("test_files/fn07.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_fn08() {
        run_test("test_files/fn08.rs", "DEBUG: U64(8)", "DEBUG: 8");
    }

    #[test]
    fn test_var01() {
        run_test("test_files/var01.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_var02() {
        run_test("test_files/var02.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_var03() {
        run_test("test_files/var03.rs", "DEBUG: Bool(true)", "DEBUG: 1");
    }

    #[test]
    fn test_var_bad01() {
        run_bad_test("test_files/var_bad01.rs", "used before being given a value");
    }

    #[test]
    fn test_infer01() {
        run_test("test_files/infer01.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_if01() {
        run_test("test_files/if01.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_if02() {
        run_test("test_files/if02.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_if03() {
        run_test("test_files/if03.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_if04() {
        run_test("test_files/if04.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_if05() {
        run_test("test_files/if05.rs", "DEBUG: U64(5)", "DEBUG: 5");
    }

    #[test]
    fn test_if06() {
        run_test("test_files/if06.rs", "DEBUG: U64(6)", "DEBUG: 6");
    }

    #[test]
    fn test_mod01() {
        run_test("test_files/mod01.rs", "DEBUG: U64(1)", "DEBUG: 1");
    }

    #[test]
    fn test_mod02() {
        run_test("test_files/mod02.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_mod03() {
        run_test("test_files/mod03.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_mod04() {
        run_test("test_files/mod04.rs", "DEBUG: U64(4)", "DEBUG: 4");
    }

    #[test]
    fn test_mod05() {
        run_test("test_files/mod05.rs", "DEBUG: U64(7)", "DEBUG: 7");
    }

    #[test]
    fn test_mod06() {
        run_test("test_files/mod06.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_mod_bad01() {
        run_bad_test("test_files/mod_bad01.rs", "not found in module");
    }

    #[test]
    fn test_scope01() {
        run_test("test_files/scope01.rs", "DEBUG: U64(3)", "DEBUG: 3");
    }

    #[test]
    fn test_scope02() {
        run_test("test_files/scope02.rs", "DEBUG: U64(2)", "DEBUG: 2");
    }

    #[test]
    fn test_scope_bad01() {
        run_bad_test("test_files/scope_bad01.rs", "Can not call function");
    }

    #[test]
    fn test_while01() {
        run_test("test_files/while01.rs", "DEBUG: U64(10)", "DEBUG: 10");
    }
}
