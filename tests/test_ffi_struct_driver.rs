use aether::pipeline::{CompilationPipeline, CompileOptions};
use std::path::PathBuf;

fn main() {
    // Create compilation options
    let mut options = CompileOptions::default();
    options.output = Some(PathBuf::from("test_ffi_struct_simple"));
    options.keep_intermediates = true;
    
    // Create pipeline
    let pipeline = CompilationPipeline::new(options);
    
    // Compile the test file
    match pipeline.compile_files(&[PathBuf::from("test_ffi_struct_simple.aether")]) {
        Ok(result) => {
            println!("Compilation successful!");
            println!("Output: {:?}", result.executable_path);
        }
        Err(e) => {
            eprintln!("Compilation failed: {:?}", e);
            std::process::exit(1);
        }
    }
}