# AetherScript Language Reference (V2)

## 1. Introduction

AetherScript V2 is a modern systems programming language designed for high performance, memory safety, and clarity. It features a C-family syntax with strong static typing, an ownership system, and built-in support for contracts and asynchronous programming.

## 2. Modules and Imports

Code is organized into modules.

```aether
module MyModule {
    import std.io;
    import std.collections;

    // Module contents...
}
```

## 3. Variables and Constants

Variables are declared with `let` (immutable) or `var` (mutable). Constants use `const`.

```aether
// Immutable variable
let x: Int = 42;

// Mutable variable
var y: Int = 10;
y = 20;

// Constant
const PI: Float = 3.14159;
```

## 4. Types

### Primitive Types
- `Int`: Signed integer (platform dependent or specified like `Int32`, `Int64`).
- `Float`: Floating point number.
- `Bool`: Boolean (`true` or `false`).
- `String`: UTF-8 string.
- `Void`: Unit type.

### Compound Types
- `Array<T>`: Dynamic array.
- `Map<K, V>`: Hash map.
- `Pointer<T>`: Raw pointer.

## 5. Structures

Structs define named compound types with fields.

```aether
struct Point {
    x: Int;
    y: Int;
}

// Construction
let p: Point = Point { x: 10, y: 20 };

// Field Access
let x: Int = p.x;
```

## 6. Enumerations

Enums define types that can hold one of several variants, optionally with associated data.

```aether
enum Option {
    case Some(Int);
    case None;
}

enum Color {
    case Red;
    case Green;
    case Blue;
}

// Usage
let opt: Option = Option::Some(42);
let col: Color = Color::Red;
```

## 7. Functions

Functions are declared with `func`. Arguments can be labeled.

```aether
func add(a: Int, b: Int) -> Int {
    return {a + b};
}

// Calling
let sum: Int = add(a: 10, b: 20);
```

### Contracts
Functions can have preconditions and postconditions.

```aether
@requires({b != 0}, "Division by zero")
@ensures({result != 0})
func safe_divide(a: Int, b: Int) -> Int {
    return {a / b};
}
```

## 8. Expressions

**Important:** Binary operations and comparisons MUST be enclosed in braces `{}`.

```aether
let sum: Int = {a + b};
let is_positive: Bool = {x > 0};
let complex: Int = {{a * b} + c};
```

## 9. Control Flow

### If / When
Standard `if` statements are supported, though `when` is preferred for complex conditions.

```aether
if ({x > 0}) {
    print("Positive");
} else {
    print("Non-positive");
}

// When syntax (pattern matching on conditions)
when ({x > 0}) {
    // ...
}
```

### Loops
`while` and `for` loops.

```aether
while ({i < 10}) {
    i = {i + 1};
}

for item in items {
    process(item);
}
```

### Pattern Matching
`match` statements for enums and values.

```aether
match opt {
    Option::Some(val) => { return val; }
    Option::None => { return 0; }
}

match x {
    0 => { print("Zero"); }
    _ => { print("Other"); }
}
```

Guards are supported:
```aether
match n {
    x if {x < 0} => { return -1; }
    _ => { return 0; }
}
```

## 10. Error Handling

AetherScript uses `Result` types or exceptions (implementation dependent). Standard `try-catch` syntax is reserved.

## 11. Asynchronous I/O

Asynchronous operations are handled via `concurrent` blocks.

```aether
concurrent {
    let data: String = read_file("file.txt");
    print(data);
}
```

## 12. Foreign Function Interface (FFI)

External functions can be declared using `@extern`.

```aether
@extern(library: "libc")
func malloc(size: Int) -> Pointer<Void>;
```
