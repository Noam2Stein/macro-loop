# macro_loop

The `macro_loop` Rust crate provides structured macro logic, with loops, conditionals and variables over fragments,
to make complex code generation readable and composable.

For-loops emit their body per value:

```rust
use macro_loop::macro_loop;

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

If-statements emit their body only if their condition is met:

```rust
use macro_loop::macro_loop;

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
    not_equal!(a b); // prints
    not_equal!(b b); // doesn't print
}
```

Multiple parameters in for-loops emit their body per combination of values:

```rust
use macro_loop::macro_loop;

macro_loop! {
  @for F in [Bool, Int, Float], I in [Bool, Int, Float] {
    @if @F != @I {
      struct @[@F To @I];
    }
  }
}

// outputs:
// struct BoolToInt;
// struct BoolToFloat;
// struct IntToBool;
// struct IntToFloat;
// struct FloatToBool;
// struct FloatToInt;
```

Let-statements give names to fragments:

```rust
use macro_loop::macro_loop;

#[derive(Debug, Clone, Copy)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

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

List fragments can be seperated:

```rust
use macro_loop::macro_loop;

fn main() {
    macro_loop! {
        @for [KEY, VALUE] in [["one", 1], ["two", 2], ["three", 3]] {
            println!("{} => {}", @KEY, @VALUE);
        }

        @let [A, B] = ['a', 'b'];

        println!("{}, {}", @A, @B);
    }
}
```
