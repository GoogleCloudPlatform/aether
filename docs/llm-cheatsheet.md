# AetherScript V2 Cheat Sheet

## File Structure
```aether
module name;
// imports, structs, enums, functions
func main() -> Int { return 0; }
```

## Variables
```aether
let x: Int = 10;        // immutable
var y: Int = 20;        // mutable
const Z: Int = 30;      // constant
```

## Types
`Int` `Int32` `Int64` `Float` `Float32` `Float64` `Bool` `String` `Char` `Void`
`Array<T>` `Map<K,V>` `*T` (pointer) `*mut T` (mutable pointer)
`^T` (owned) `&T` (borrowed) `&mut T` (mutable borrow) `~T` (shared)

## Functions
```aether
func name(a: Int, b: Int) -> Int {
    return {a + b};  // BRACES REQUIRED for binary ops
}
```

## CRITICAL: Braced Expressions
```aether
return {a + b};           // CORRECT
let x: Int = {y * 2};     // CORRECT
return a + b;             // WRONG!
```

## Control Flow
```aether
when {condition} {        // NOT "if"!
    // ...
} else {
    // ...
}

while {condition} { }

for item in collection { }
for i in 0..10 { }        // exclusive range
for i in 0..=10 { }       // inclusive range
```

## Structs
```aether
struct Point { x: Int, y: Int, }
let p: Point = Point { x: 10, y: 20 };
let x: Int = p.x;
```

## Enums
```aether
enum Option { Some(Int), None, }
let x: Option = Option::Some(42);  // Use ::
let y: Option = Option::None;
```

## Pattern Matching
```aether
match value {
    1 => { result1 }
    2 => { result2 }
    _ => { default }
}

match opt {
    Option::Some(v) => { use(v); }
    Option::None => { handle_none(); }
}
```

## Lambdas
```aether
let f = (a: Int) => {a * 2};
let f = (a: Int) -> Int => {a * 2};
let f = [capture](a: Int) => {a + capture};
let f = [&ref, &mut mref](a: Int) => { /* ... */ };
```

## FFI
```aether
@extern("C")
func puts(s: *Char) -> Int;
```

## Common Mistakes
| Wrong | Correct |
|-------|---------|
| `return a + b;` | `return {a + b};` |
| `if cond { }` | `when cond { }` |
| `let x = 10;` | `let x: Int = 10;` |
| `Color::Red` | `Color::Red` (enum needs `::`) |
| Missing `;` | All statements need `;` |

## Keywords
`module` `func` `let` `var` `const` `struct` `enum` `when` `else` `while` `for` `in` `match` `return` `break` `continue` `true` `false` `mut`

## Operators
Arithmetic: `+` `-` `*` `/` `%`
Comparison: `==` `!=` `<` `>` `<=` `>=`
Logical: `&&` `||` `!`
Range: `..` `..=`
Access: `.` `::`
