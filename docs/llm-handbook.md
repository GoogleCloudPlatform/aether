# AetherScript V2 Language Handbook for LLMs

This document provides a comprehensive reference for AI agents to read and write AetherScript V2 code. AetherScript is a systems programming language with Swift/Rust-like syntax, ownership semantics, and pattern matching.

## Quick Reference Card

```
File extension: .aes
Comments: // single line, /// doc comments
Module declaration: module name;
Function: func name(param: Type) -> ReturnType { }
Variables: let x: Int = 5; (immutable), var y: Int = 5; (mutable)
Expressions in braces: return {a + b};
```

---

## 1. Program Structure

Every AetherScript program starts with a module declaration:

```aether
// File: main.aes
module main;

func main() -> Int {
    return 0;
}
```

### Module Declaration Styles

```aether
// File-scoped module (preferred)
module my_module;

// Inline module (for nested modules)
module nested {
    func helper() -> Int { return 0; }
}
```

---

## 2. Variables and Constants

### Immutable Variables (let)

```aether
let x: Int = 10;
let name: String = "hello";
let flag: Bool = true;
```

### Mutable Variables (var)

```aether
var counter: Int = 0;
counter = counter + 1;
```

### Constants

```aether
const MAX_SIZE: Int = 100;
const PI: Float = 3.14159;
```

---

## 3. Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | Integer (platform-sized) | `42` |
| `Int32` | 32-bit integer | `42` |
| `Int64` | 64-bit integer | `42` |
| `Float` | Floating point (platform) | `3.14` |
| `Float32` | 32-bit float | `3.14` |
| `Float64` | 64-bit float | `3.14` |
| `Bool` | Boolean | `true`, `false` |
| `String` | String | `"hello"` |
| `Char` | Character | `'a'` |
| `Void` | No value | - |

---

## 4. Functions

### Basic Function

```aether
func add(a: Int, b: Int) -> Int {
    return {a + b};
}
```

### Function with No Return Value

```aether
func print_message(msg: String) -> Void {
    // ... print logic
}
```

### Function Calling

```aether
let result: Int = add(10, 20);
```

### IMPORTANT: Braced Expressions

**AetherScript requires braces `{}` around binary expressions in return statements and assignments:**

```aether
// CORRECT
return {a + b};
let sum: Int = {x + y};
let product: Int = {a * b};

// INCORRECT - will not parse
return a + b;
let sum: Int = x + y;
```

Simple literals and single variables do NOT need braces:

```aether
// These are fine without braces
return 42;
return x;
let y: Int = x;
```

---

## 5. Control Flow

### If/Else (when/else)

```aether
func max(a: Int, b: Int) -> Int {
    when {a > b} {
        return a;
    } else {
        return b;
    }
}
```

### While Loops

```aether
var i: Int = 0;
while {i < 10} {
    i = {i + 1};
}
```

### For-Each Loops

```aether
for item in collection {
    // process item
}

// With type annotation
for x: Int in numbers {
    // process x
}
```

### Break and Continue

```aether
while true {
    when condition {
        break;
    }
    when skip_condition {
        continue;
    }
}
```

---

## 6. Structs

### Definition

```aether
struct Point {
    x: Int,
    y: Int,
}
```

### Instantiation

```aether
let p: Point = Point { x: 10, y: 20 };
```

### Field Access

```aether
let x_coord: Int = p.x;
```

### Nested Structs

```aether
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

let rect: Rectangle = Rectangle {
    top_left: Point { x: 0, y: 0 },
    bottom_right: Point { x: 100, y: 100 },
};
```

---

## 7. Enums

### Simple Enum

```aether
enum Color {
    Red,
    Green,
    Blue,
}
```

### Enum with Associated Data

```aether
enum Option {
    Some(Int),
    None,
}

enum Result {
    Ok(Int),
    Err(String),
}
```

### Using Enums

```aether
let color: Color = Color::Red;
let maybe_value: Option = Option::Some(42);
let nothing: Option = Option::None;
```

---

## 8. Pattern Matching

### Basic Match

```aether
match x {
    1 => { return "one"; }
    2 => { return "two"; }
    _ => { return "other"; }
}
```

### Matching Enums

```aether
match option_value {
    Option::Some(value) => {
        return value;
    }
    Option::None => {
        return 0;
    }
}
```

### Match with Guards

```aether
match x {
    n if {n > 0} => { return "positive"; }
    n if {n < 0} => { return "negative"; }
    _ => { return "zero"; }
}
```

### Struct Destructuring

```aether
match point {
    Point { x: 0, y: 0 } => { return "origin"; }
    Point { x, y } => { return "other point"; }
}
```

---

## 9. Arrays and Collections

### Fixed-Size Arrays

```aether
let numbers: Array<Int> = [1, 2, 3, 4, 5];
let first: Int = numbers[0];
```

### Array Operations

```aether
let length: Int = numbers.len();
```

### Maps

```aether
let scores: Map<String, Int> = {};
// Map operations via methods
```

---

## 10. Ownership and References

### Ownership Sigils

| Sigil | Meaning | Example |
|-------|---------|---------|
| `^` | Owned | `^String` |
| `&` | Borrowed (immutable) | `&String` |
| `&mut` | Borrowed (mutable) | `&mut String` |
| `~` | Shared (reference counted) | `~String` |

### Function Parameters with Ownership

```aether
// Takes ownership
func consume(s: ^String) -> Void { }

// Borrows immutably
func read(s: &String) -> Int { }

// Borrows mutably
func modify(s: &mut String) -> Void { }
```

### Pointers

```aether
let ptr: *Int = &x;           // Immutable pointer
let mut_ptr: *mut Int = &mut y;  // Mutable pointer
```

---

## 11. Lambda Expressions

### Basic Lambda

```aether
let add = (a: Int, b: Int) => {a + b};
```

### Lambda with Return Type

```aether
let multiply = (a: Int, b: Int) -> Int => {a * b};
```

### Lambda with Captures

```aether
let x: Int = 10;

// Capture by value
let add_x = [x](a: Int) => {a + x};

// Capture by reference
let use_ref = [&x](a: Int) => {a + x};

// Capture by mutable reference
let mutate = [&mut counter]() => { counter = {counter + 1}; };

// Multiple captures
let complex = [x, &y, &mut z](a: Int) => { /* ... */ };
```

---

## 12. Method Calls

```aether
let result: Int = obj.method();
let chained: Int = obj.first().second().third();
let with_args: Int = obj.method(arg1, arg2);
```

---

## 13. Range Expressions

```aether
// Exclusive range: 0, 1, 2, ..., 9
for i in 0..10 { }

// Inclusive range: 0, 1, 2, ..., 10
for i in 0..=10 { }
```

---

## 14. Annotations/Attributes

### External Functions (FFI)

```aether
@extern("C")
func puts(s: *Char) -> Int;

@extern("C")
func printf(format: *Char) -> Int;
```

### Function Metadata

```aether
@requires("x > 0")
@ensures("result >= 0")
func sqrt(x: Float) -> Float {
    // implementation
}
```

---

## 15. Common Patterns

### Option Handling

```aether
func find_value(key: String) -> Option {
    when found {
        return Option::Some(value);
    } else {
        return Option::None;
    }
}

// Usage
match find_value("key") {
    Option::Some(v) => { use_value(v); }
    Option::None => { handle_missing(); }
}
```

### Result/Error Handling

```aether
enum Result {
    Ok(Int),
    Err(String),
}

func divide(a: Int, b: Int) -> Result {
    when {b == 0} {
        return Result::Err("division by zero");
    } else {
        return Result::Ok({a / b});
    }
}
```

### Recursive Functions

```aether
func factorial(n: Int) -> Int {
    when {n <= 1} {
        return 1;
    } else {
        return {n * factorial({n - 1})};
    }
}
```

---

## 16. Complete Example Program

```aether
// File: calculator.aes
module calculator;

struct Calculator {
    value: Int,
}

enum Operation {
    Add(Int),
    Subtract(Int),
    Multiply(Int),
    Divide(Int),
}

func create_calculator(initial: Int) -> Calculator {
    return Calculator { value: initial };
}

func apply(calc: Calculator, op: Operation) -> Calculator {
    let new_value: Int = match op {
        Operation::Add(n) => { {calc.value + n} }
        Operation::Subtract(n) => { {calc.value - n} }
        Operation::Multiply(n) => { {calc.value * n} }
        Operation::Divide(n) => {
            when {n == 0} {
                calc.value
            } else {
                {calc.value / n}
            }
        }
    };
    return Calculator { value: new_value };
}

func main() -> Int {
    let calc: Calculator = create_calculator(10);
    let calc2: Calculator = apply(calc, Operation::Add(5));
    let calc3: Calculator = apply(calc2, Operation::Multiply(2));
    return calc3.value;  // Returns 30
}
```

---

## 17. Common Mistakes to Avoid

### 1. Missing Braces Around Binary Expressions

```aether
// WRONG
return a + b;
let x: Int = y * 2;

// CORRECT
return {a + b};
let x: Int = {y * 2};
```

### 2. Using `if` Instead of `when`

```aether
// WRONG
if condition { }

// CORRECT
when condition { }
```

### 3. Missing Type Annotations

```aether
// WRONG (types required)
let x = 10;

// CORRECT
let x: Int = 10;
```

### 4. Wrong Enum Syntax

```aether
// WRONG
let c = Red;

// CORRECT
let c: Color = Color::Red;
```

### 5. Forgetting Semicolons

```aether
// WRONG
let x: Int = 10
return x

// CORRECT
let x: Int = 10;
return x;
```

---

## 18. Keywords Reference

| Keyword | Usage |
|---------|-------|
| `module` | Module declaration |
| `func` | Function declaration |
| `let` | Immutable variable |
| `var` | Mutable variable |
| `const` | Constant |
| `struct` | Struct definition |
| `enum` | Enum definition |
| `when` | Conditional (if) |
| `else` | Else branch |
| `while` | While loop |
| `for` | For-each loop |
| `in` | Iterator keyword |
| `match` | Pattern matching |
| `return` | Return statement |
| `break` | Break loop |
| `continue` | Continue loop |
| `true` | Boolean true |
| `false` | Boolean false |
| `mut` | Mutable modifier |

---

## 19. Operators

### Arithmetic
`+`, `-`, `*`, `/`, `%`

### Comparison
`==`, `!=`, `<`, `>`, `<=`, `>=`

### Logical
`&&`, `||`, `!`

### Assignment
`=`

### Range
`..` (exclusive), `..=` (inclusive)

### Member Access
`.` (field/method), `::` (enum variant/associated)

---

## 20. FFI Example (Calling C)

```aether
module ffi_example;

@extern("C")
func puts(s: *Char) -> Int;

@extern("C")
func printf(format: *Char, value: Int) -> Int;

func main() -> Int {
    puts("Hello from AetherScript!");
    printf("The answer is: %d\n", 42);
    return 0;
}
```

---

## Summary for Code Generation

When generating AetherScript V2 code:

1. **Always start with** `module name;`
2. **Always use braces** around binary expressions: `{a + b}`
3. **Use `when`** for conditionals, not `if`
4. **Always specify types** for variables and parameters
5. **Use `::` for enum variants**: `Option::Some(x)`
6. **End statements with semicolons**
7. **Use `=>` in match arms**
8. **Lambdas use** `(params) => body` syntax
9. **Captures use** `[x, &y]` before lambda params
