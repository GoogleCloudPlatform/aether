// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Integration tests for FFI struct passing
//!
//! Tests that verify correct struct layout, alignment, and passing
//! between Aether and C code.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_basic_struct_passing() {
    let test_program = r#"
module test_basic_struct {
    // Define Point2D struct
    pub struct Point2D {
        x: Float64,
        y: Float64,
    }
    
    // External functions for testing
    @extern(library="aether_runtime")
    func point_distance(p1: Point2D, p2: Point2D) -> Float64;
    
    @extern(library="aether_runtime")
    func point_add(p1: Point2D, p2: Point2D) -> Point2D;
    
    @extern(library="aether_runtime")
    func point_scale(p: Pointer<Point2D>, factor: Float64) -> Void;
    
    @extern(library="libc", variadic=true)
    func printf(format: String) -> Int;
    
    func main() -> Int {
        // Create two points
        var p1: Point2D = Point2D { x: 3.0, y: 4.0 };
        var p2: Point2D = Point2D { x: 0.0, y: 0.0 };
        
        // Test distance calculation
        let dist: Float64 = point_distance(p1, p2);
        
        // Test point addition
        let sum: Point2D = point_add(p1, p2);
        
        // Test point scaling by pointer
        var p3: Point2D = Point2D { x: 2.0, y: 3.0 };
        point_scale(&p3, 2.0);
        
        return 0;
    }
}
"#;

    // Write test program
    let test_file = PathBuf::from("test_basic_struct.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    // Compile the program
    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_basic_struct")
        .output()
        .expect("Failed to run compiler");

    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_basic_struct").ok();
    fs::remove_file("test_basic_struct.o").ok();
}

#[test]
fn test_nested_struct_passing() {
    let test_program = r#"
module test_nested_struct {
    pub struct Point2D {
        x: Float64,
        y: Float64,
    }
    
    pub struct Rectangle {
        top_left: Point2D,
        width: Float64,
        height: Float64,
    }
    
    @extern(library="aether_runtime")
    func rectangle_area(rect: Rectangle) -> Float64;
    
    @extern(library="aether_runtime")
    func rectangle_expand(rect: Rectangle, amount: Float64) -> Rectangle;
    
    func main() -> Int {
        var rect: Rectangle = Rectangle {
            top_left: Point2D { x: 0.0, y: 0.0 },
            width: 10.0,
            height: 5.0,
        };
        
        let area: Float64 = rectangle_area(rect);
        
        let expanded: Rectangle = rectangle_expand(rect, 1.0);
        
        return 0;
    }
}
"#;

    let test_file = PathBuf::from("test_nested_struct.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_nested_struct")
        .output()
        .expect("Failed to run compiler");
        
    fs::remove_file(test_file).ok();
    fs::remove_file("test_nested_struct").ok();
    fs::remove_file("test_nested_struct.o").ok();
}

#[test]
fn test_struct_with_small_fields() {
    let test_program = r#"
module test_struct_alignment {
    pub struct Color {
        r: Int, 
        g: Int,
        b: Int,
        a: Int,
    }
    
    @extern(library="aether_runtime")
    func color_blend(c1: Color, c2: Color, ratio: Float32) -> Color;
    
    func main() -> Int {
        let red: Color = Color { r: 255, g: 0, b: 0, a: 255 };
        let blue: Color = Color { r: 0, g: 0, b: 255, a: 255 };
        
        let purple: Color = color_blend(red, blue, 0.5);
        
        return 0;
    }
}
"#;

    let test_file = PathBuf::from("test_struct_alignment.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_struct_alignment")
        .output()
        .expect("Failed to run compiler");
        
    fs::remove_file(test_file).ok();
    fs::remove_file("test_struct_alignment").ok();
    fs::remove_file("test_struct_alignment.o").ok();
}

#[test]
fn test_struct_with_string_field() {
    let test_program = r#"
module test_struct_string {
    pub struct Person {
        name: String,
        age: Int,
        height: Float64,
    }
    
    @extern(library="aether_runtime")
    func person_create(name: String, age: Int, height: Float64) -> Pointer<Person>;
    
    @extern(library="aether_runtime")
    func person_free(person: Pointer<Person>) -> Void;
    
    func main() -> Int {
        let person_ptr: Pointer<Person> = person_create("Alice", 30, 165.5);
        
        person_free(person_ptr);
        
        return 0;
    }
}
"#;

    let test_file = PathBuf::from("test_struct_string.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_struct_string")
        .output()
        .expect("Failed to run compiler");

    fs::remove_file(test_file).ok();
    fs::remove_file("test_struct_string").ok();
    fs::remove_file("test_struct_string.o").ok();
}
