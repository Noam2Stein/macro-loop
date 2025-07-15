use macro_loop::macro_loop;

fn main() {}

macro_loop! {
    @for A in 0..10, B in 0..10 {
        struct @[T @A @B] {}
    }
}
