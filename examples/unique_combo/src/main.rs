#![allow(dead_code)]

use macro_loop::macro_loop;

fn main() {}

macro_loop! {
    @for A in 0..5, B in 0..5 {
        @if @A != @B {
            struct @[T @A @B];
        }
    }
}
