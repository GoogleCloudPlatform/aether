#[cfg(test)]
mod ownership_integration_tests {
    use std::process::Command;
    use std::fs;
    use std::path::Path;
    
    fn compile_and_run(source_file: &str, expected_exit_code: i32) {
        let output_file = source_file.replace(".aether", "");
        
        // Compile the source file
        let compile_result = Command::new("cargo")
            .args(&["run", "--", source_file, "-o", &output_file])
            .output()
            .expect("Failed to execute compiler");
        
        if !compile_result.status.success() {
            panic!(
                "Compilation failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&compile_result.stdout),
                String::from_utf8_lossy(&compile_result.stderr)
            );
        }
        
        // Run the compiled program
        let run_result = Command::new(&format!("./{}", output_file))
            .output()
            .expect("Failed to execute compiled program");
        
        assert_eq!(
            run_result.status.code().unwrap_or(-1),
            expected_exit_code,
            "Program exited with unexpected code.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&run_result.stdout),
            String::from_utf8_lossy(&run_result.stderr)
        );
        
        // Clean up
        fs::remove_file(&output_file).ok();
    }
    
    #[test]
    fn test_string_cleanup() {
        let source = r#"
module string_cleanup_test {
    func create_and_drop_strings() {
        let s1: ^string = "String 1";
        let s2: ^string = "String 2";
        let s3: ^string = "String 3";
        // All strings should be cleaned up when function exits
    }
    
    func main() -> int {
        create_and_drop_strings();
        // Memory should be freed
        return 0;
    }
}
"#;
        
        let test_file = "test_string_cleanup.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
    
    #[test]
    fn test_array_cleanup() {
        let source = r#"
module array_cleanup_test {
    func create_and_drop_arrays() {
        let arr1: ^[int; 10] = [0; 10];
        let arr2: ^[int; 5] = [1, 2, 3, 4, 5];
        // Arrays should be cleaned up when function exits
    }
    
    func main() -> int {
        create_and_drop_arrays();
        return 0;
    }
}
"#;
        
        let test_file = "test_array_cleanup.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
    
    #[test]
    fn test_map_cleanup() {
        let source = r#"
module map_cleanup_test {
    func create_and_drop_maps() {
        let mut m1: ^map<string, int> = {};
        m1["key1"] = 100;
        m1["key2"] = 200;
        
        let mut m2: ^map<int, string> = {};
        m2[1] = "value1";
        m2[2] = "value2";
        // Maps should be cleaned up when function exits
    }
    
    func main() -> int {
        create_and_drop_maps();
        return 0;
    }
}
"#;
        
        let test_file = "test_map_cleanup.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
    
    #[test]
    fn test_early_return_cleanup() {
        let source = r#"
module early_return_cleanup_test {
    func test_early_return(flag: bool) -> int {
        let s1: ^string = "String 1";
        let arr: ^[int; 5] = [1, 2, 3, 4, 5];
        
        if flag {
            // s1 and arr should be cleaned up before return
            return 1;
        }
        
        let s2: ^string = "String 2";
        // All should be cleaned up
        return 0;
    }
    
    func main() -> int {
        let result1 = test_early_return(true);
        let result2 = test_early_return(false);
        return 0;
    }
}
"#;
        
        let test_file = "test_early_return_cleanup.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
    
    #[test]
    fn test_shared_ownership_refcount() {
        let source = r#"
module shared_ownership_test {
    func use_shared_string(s: ~string) -> int {
        // Just use the shared string
        return 0;
    }
    
    func test_shared() {
        let s1: ~string = ~"Shared string";
        let s2: ~string = s1;  // Ref count = 2
        let s3: ~string = s2;  // Ref count = 3
        
        use_shared_string(s1);
        use_shared_string(s2);
        use_shared_string(s3);
        // String should only be freed when last reference goes out of scope
    }
    
    func main() -> int {
        test_shared();
        return 0;
    }
}
"#;
        
        let test_file = "test_shared_ownership.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
    
    #[test]
    fn test_nested_scopes_cleanup() {
        let source = r#"
module nested_scopes_test {
    func main() -> int {
        let outer: ^string = "Outer string";
        
        {
            let inner1: ^string = "Inner string 1";
            {
                let inner2: ^string = "Inner string 2";
                // inner2 cleaned up here
            }
            // inner1 cleaned up here
        }
        
        // outer cleaned up at function exit
        return 0;
    }
}
"#;
        
        let test_file = "test_nested_scopes.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
    
    #[test]
    fn test_loop_cleanup() {
        let source = r#"
module loop_cleanup_test {
    func main() -> int {
        let mut i = 0;
        while i < 10 {
            let s: ^string = "Loop string";
            let arr: ^[int; 3] = [i, i+1, i+2];
            // Both should be cleaned up at end of each iteration
            i = i + 1;
        }
        return 0;
    }
}
"#;
        
        let test_file = "test_loop_cleanup.aether";
        fs::write(test_file, source).expect("Failed to write test file");
        
        compile_and_run(test_file, 0);
        
        fs::remove_file(test_file).ok();
    }
}