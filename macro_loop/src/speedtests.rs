use std::{hint::black_box, time::Instant};

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;

use crate::macro_loop2;

#[test]
fn test_swizzle_macro() {
    speedtest(quote! {
        struct Vec2 {
            x: f32,
            y: f32,
        }

        struct Vec3 {
            x: f32,
            y: f32,
            z: f32,
        }

        struct Vec4 {
            x: f32,
            y: f32,
            z: f32,
            w: f32,
        }

        @for N in 2..=4 {
            @let components = [x, y, z, w][0..N];

            @for x in @components {
                impl @[Vec@N] {
                    fn @[@x](self) -> f32 {
                        self.@x
                    }
                }
            }

            @for x in @components, y in @components {
                impl @[Vec@N] {
                    fn @[@x @y](self) -> Vec2 {
                        Vec2 {
                            x: self.@x,
                            y: self.@y,
                        }
                    }
                }
            }

            @for x in @components, y in @components, z in @components {
                impl @[Vec@N] {
                    fn @[@x @y @z](self) -> Vec3 {
                        Vec3 {
                            x: self.@x,
                            y: self.@y,
                            z: self.@z,
                        }
                    }
                }
            }

            @for x in @components, y in @components, z in @components, w in @components {
                impl @[Vec@N] {
                    fn @[@x @y @z @w](self) -> Vec4 {
                        Vec4 {
                            x: self.@x,
                            y: self.@y,
                            z: self.@z,
                            w: self.@w,
                        }
                    }
                }
            }
        }
    });
}

fn speedtest(input: TokenStream) {
    let start = Instant::now();

    let _ = black_box(macro_loop2.parse2(input));

    let duration = start.elapsed();

    println!("the macro took {}ns", duration.as_nanos());
}
