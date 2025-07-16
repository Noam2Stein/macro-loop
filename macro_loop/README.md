# macro_loop

`macro_loop` is a macro for writing repetitive Rust code using loops, conditionals, and bindings.

This is similar to the `paste`, `seq-macro` and `tt-call` crates,
but attempts to make it more readable and scalable.

`macro_loop` supports:
- for loops - loops over list values which can be literals or identifiers.
- if statements - declares a condition over fragments and only emits the body if the condition is met.
- concat idents/strings - merges idents/string into one anywhere in the code.

These features together allow for simple, readable and scalable macro logic.

### Examples

Concat:

```rust
macro_loop! {
    struct @[T ype];
}

// outputs:
// struct Type;
```

For loops:

```rust
macro_loop! {
    @for N in 2..=4 {
        struct @[Vec @N];
    }
}

// outputs:
// struct Vec2;
// struct Vec3;
// struct Vec4;
```
```rust
macro_loop! {
    // Generates combinations of T x N
    @for N in 2..=4, T in [bool, i32, f32] {
        struct @[@T x @N];
    }
}

// outputs:
// struct boolx2;
// struct boolx3;
// struct boolx4;
// struct i32x2;
// ...
```

If statements:

```rust
// Only prints if `$a` and `$b` are NOT equal.
macro_rules! not_equal {
    ($a:ident $b:ident) => {
        macro_loop! {
            @if $a != $b {
                println!("{}", stringify!($a != $b))
            }
        }
    };
}

fn main() {
    not_equal!(a a); // doesn't print
    not_equal!(a b); // does print
    not_equal!(b b); // doesn't print
}
```

Let statements:

```rust
#[derive(Debug, Clone, Copy)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

// Declare all swizzle fns
impl Vec4 {
    macro_loop! {
        @let components = [x, y, z, w];

        @for X in @components, Y in @components, Z in @components, W in @components {
            pub fn @[@X @Y @Z @W](self) -> Vec4 {
                Vec4 { x: self.@X, y: self.@Y, z: self.@Z, w: self.@W }
            }
        }
    }
}
```

Patterns:

```rust
fn main() {
    macro_loop! {
        @for [KEY, VALUE] in [["one", 1], ["two", 2], ["three", 3]] {
            println!("{} => {}", @KEY, @VALUE);
        }
    }
}
```