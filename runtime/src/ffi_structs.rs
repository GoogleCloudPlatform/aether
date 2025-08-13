//! FFI struct passing tests and utilities
//! 
//! This module provides test structures and functions to verify correct
//! struct passing between Aether and C code.

use std::ffi::{c_char, c_int, c_void};
use std::mem;
use std::ptr;

/// Simple struct with primitive fields
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// Struct with mixed types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub top_left: Point2D,
    pub width: f64,
    pub height: f64,
}

/// Struct with string field
#[repr(C)]
#[derive(Debug)]
pub struct Person {
    pub name: *const c_char,
    pub age: c_int,
    pub height: f64,
}

/// Complex nested struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ColoredRectangle {
    pub rect: Rectangle,
    pub fill_color: Color,
    pub border_color: Color,
    pub border_width: f32,
}

/// Struct with array field
#[repr(C)]
#[derive(Debug)]
pub struct Vector3Array {
    pub count: c_int,
    pub data: *mut f64, // Points to array of x,y,z triplets
}

/// Struct with union-like variant (using tagged union pattern)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ShapeKind {
    pub tag: c_int, // 0=circle, 1=rectangle, 2=triangle
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ShapeData {
    pub circle: Circle,
    pub rectangle: Rectangle,
    pub triangle: Triangle,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Shape {
    pub kind: ShapeKind,
    pub data: ShapeData,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub center: Point2D,
    pub radius: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Point2D,
    pub b: Point2D,
    pub c: Point2D,
}

// FFI functions for testing struct passing

/// Test passing struct by value
#[no_mangle]
pub unsafe extern "C" fn point_distance(p1: Point2D, p2: Point2D) -> f64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx * dx + dy * dy).sqrt()
}

/// Test returning struct by value
#[no_mangle]
pub unsafe extern "C" fn point_add(p1: Point2D, p2: Point2D) -> Point2D {
    Point2D {
        x: p1.x + p2.x,
        y: p1.y + p2.y,
    }
}

/// Test passing struct by pointer (mutable)
#[no_mangle]
pub unsafe extern "C" fn point_scale(p: *mut Point2D, factor: f64) {
    if !p.is_null() {
        (*p).x *= factor;
        (*p).y *= factor;
    }
}

/// Test passing nested struct
#[no_mangle]
pub unsafe extern "C" fn rectangle_area(rect: Rectangle) -> f64 {
    rect.width * rect.height
}

/// Test returning nested struct
#[no_mangle]
pub unsafe extern "C" fn rectangle_expand(rect: Rectangle, amount: f64) -> Rectangle {
    Rectangle {
        top_left: Point2D {
            x: rect.top_left.x - amount,
            y: rect.top_left.y - amount,
        },
        width: rect.width + 2.0 * amount,
        height: rect.height + 2.0 * amount,
    }
}

/// Test struct with string field
#[no_mangle]
pub unsafe extern "C" fn person_create(name: *const c_char, age: c_int, height: f64) -> *mut Person {
    let person_ptr = crate::memory::aether_malloc(mem::size_of::<Person>() as c_int) as *mut Person;
    if !person_ptr.is_null() {
        (*person_ptr).name = crate::memory::aether_strdup(name);
        (*person_ptr).age = age;
        (*person_ptr).height = height;
    }
    person_ptr
}

/// Test freeing struct with string field
#[no_mangle]
pub unsafe extern "C" fn person_free(person: *mut Person) {
    if !person.is_null() {
        if !(*person).name.is_null() {
            crate::memory::aether_free((*person).name as *mut c_void);
        }
        crate::memory::aether_free(person as *mut c_void);
    }
}

/// Test struct with small fields (alignment test)
#[no_mangle]
pub unsafe extern "C" fn color_blend(c1: Color, c2: Color, ratio: f32) -> Color {
    let inv_ratio = 1.0 - ratio;
    Color {
        r: (c1.r as f32 * inv_ratio + c2.r as f32 * ratio) as u8,
        g: (c1.g as f32 * inv_ratio + c2.g as f32 * ratio) as u8,
        b: (c1.b as f32 * inv_ratio + c2.b as f32 * ratio) as u8,
        a: (c1.a as f32 * inv_ratio + c2.a as f32 * ratio) as u8,
    }
}

/// Test complex nested struct
#[no_mangle]
pub unsafe extern "C" fn colored_rectangle_contains_point(
    cr: *const ColoredRectangle,
    p: Point2D
) -> c_int {
    if cr.is_null() {
        return 0;
    }
    
    let rect = &(*cr).rect;
    if p.x >= rect.top_left.x &&
       p.x <= rect.top_left.x + rect.width &&
       p.y >= rect.top_left.y &&
       p.y <= rect.top_left.y + rect.height {
        1
    } else {
        0
    }
}

/// Test struct array operations
#[no_mangle]
pub unsafe extern "C" fn vector3_array_create(count: c_int) -> *mut Vector3Array {
    if count <= 0 {
        return ptr::null_mut();
    }
    
    let array_ptr = crate::memory::aether_malloc(mem::size_of::<Vector3Array>() as c_int) as *mut Vector3Array;
    if array_ptr.is_null() {
        return ptr::null_mut();
    }
    
    let data_size = (count as usize) * 3 * mem::size_of::<f64>();
    let data_ptr = crate::memory::aether_malloc(data_size as c_int) as *mut f64;
    
    if data_ptr.is_null() {
        crate::memory::aether_free(array_ptr as *mut c_void);
        return ptr::null_mut();
    }
    
    (*array_ptr).count = count;
    (*array_ptr).data = data_ptr;
    
    // Initialize to zero
    ptr::write_bytes(data_ptr, 0, (count as usize) * 3);
    
    array_ptr
}

#[no_mangle]
pub unsafe extern "C" fn vector3_array_free(array: *mut Vector3Array) {
    if !array.is_null() {
        if !(*array).data.is_null() {
            crate::memory::aether_free((*array).data as *mut c_void);
        }
        crate::memory::aether_free(array as *mut c_void);
    }
}

/// Test tagged union struct
#[no_mangle]
pub unsafe extern "C" fn shape_area(shape: *const Shape) -> f64 {
    if shape.is_null() {
        return 0.0;
    }
    
    match (*shape).kind.tag {
        0 => {
            // Circle
            let circle = (*shape).data.circle;
            std::f64::consts::PI * circle.radius * circle.radius
        }
        1 => {
            // Rectangle
            let rect = (*shape).data.rectangle;
            rect.width * rect.height
        }
        2 => {
            // Triangle - using cross product formula
            let tri = (*shape).data.triangle;
            let v1x = tri.b.x - tri.a.x;
            let v1y = tri.b.y - tri.a.y;
            let v2x = tri.c.x - tri.a.x;
            let v2y = tri.c.y - tri.a.y;
            0.5 * (v1x * v2y - v1y * v2x).abs()
        }
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_point_operations() {
        unsafe {
            let p1 = Point2D { x: 3.0, y: 4.0 };
            let p2 = Point2D { x: 0.0, y: 0.0 };
            
            // Test distance calculation
            let dist = point_distance(p1, p2);
            assert!((dist - 5.0).abs() < 0.0001);
            
            // Test point addition
            let p3 = point_add(p1, p2);
            assert_eq!(p3.x, 3.0);
            assert_eq!(p3.y, 4.0);
            
            // Test point scaling
            let mut p4 = Point2D { x: 2.0, y: 3.0 };
            point_scale(&mut p4, 2.0);
            assert_eq!(p4.x, 4.0);
            assert_eq!(p4.y, 6.0);
        }
    }
    
    #[test]
    fn test_rectangle_operations() {
        unsafe {
            let rect = Rectangle {
                top_left: Point2D { x: 0.0, y: 0.0 },
                width: 10.0,
                height: 5.0,
            };
            
            // Test area calculation
            let area = rectangle_area(rect);
            assert_eq!(area, 50.0);
            
            // Test expansion
            let expanded = rectangle_expand(rect, 1.0);
            assert_eq!(expanded.top_left.x, -1.0);
            assert_eq!(expanded.top_left.y, -1.0);
            assert_eq!(expanded.width, 12.0);
            assert_eq!(expanded.height, 7.0);
        }
    }
    
    #[test]
    fn test_person_struct() {
        unsafe {
            let name = CString::new("Alice").unwrap();
            let person = person_create(name.as_ptr(), 30, 165.5);
            assert!(!person.is_null());
            
            assert_eq!((*person).age, 30);
            assert!(((*person).height - 165.5).abs() < 0.0001);
            
            // Clean up
            person_free(person);
        }
    }
    
    #[test]
    fn test_color_blending() {
        unsafe {
            let red = Color { r: 255, g: 0, b: 0, a: 255 };
            let blue = Color { r: 0, g: 0, b: 255, a: 255 };
            
            let purple = color_blend(red, blue, 0.5);
            assert_eq!(purple.r, 127);
            assert_eq!(purple.g, 0);
            assert_eq!(purple.b, 127);
            assert_eq!(purple.a, 255);
        }
    }
    
    #[test]
    fn test_colored_rectangle() {
        unsafe {
            let cr = ColoredRectangle {
                rect: Rectangle {
                    top_left: Point2D { x: 0.0, y: 0.0 },
                    width: 10.0,
                    height: 10.0,
                },
                fill_color: Color { r: 255, g: 0, b: 0, a: 255 },
                border_color: Color { r: 0, g: 0, b: 0, a: 255 },
                border_width: 1.0,
            };
            
            // Test point containment
            let p1 = Point2D { x: 5.0, y: 5.0 };
            let p2 = Point2D { x: 15.0, y: 5.0 };
            
            assert_eq!(colored_rectangle_contains_point(&cr, p1), 1);
            assert_eq!(colored_rectangle_contains_point(&cr, p2), 0);
        }
    }
    
    #[test]
    fn test_vector3_array() {
        unsafe {
            let array = vector3_array_create(3);
            assert!(!array.is_null());
            assert_eq!((*array).count, 3);
            assert!(!(*array).data.is_null());
            
            // Set some values
            let data = (*array).data;
            *data.add(0) = 1.0; // x1
            *data.add(1) = 2.0; // y1
            *data.add(2) = 3.0; // z1
            
            // Verify values
            assert_eq!(*data.add(0), 1.0);
            assert_eq!(*data.add(1), 2.0);
            assert_eq!(*data.add(2), 3.0);
            
            // Clean up
            vector3_array_free(array);
        }
    }
    
    #[test]
    fn test_shape_union() {
        unsafe {
            // Test circle
            let mut circle_shape = Shape {
                kind: ShapeKind { tag: 0 },
                data: ShapeData {
                    circle: Circle {
                        center: Point2D { x: 0.0, y: 0.0 },
                        radius: 5.0,
                    }
                },
            };
            
            let circle_area = shape_area(&circle_shape);
            let expected_area = std::f64::consts::PI * 25.0;
            assert!((circle_area - expected_area).abs() < 0.0001);
            
            // Test rectangle
            circle_shape.kind.tag = 1;
            circle_shape.data.rectangle = Rectangle {
                top_left: Point2D { x: 0.0, y: 0.0 },
                width: 4.0,
                height: 3.0,
            };
            
            let rect_area = shape_area(&circle_shape);
            assert_eq!(rect_area, 12.0);
            
            // Test triangle
            circle_shape.kind.tag = 2;
            circle_shape.data.triangle = Triangle {
                a: Point2D { x: 0.0, y: 0.0 },
                b: Point2D { x: 4.0, y: 0.0 },
                c: Point2D { x: 0.0, y: 3.0 },
            };
            
            let tri_area = shape_area(&circle_shape);
            assert_eq!(tri_area, 6.0);
        }
    }
}