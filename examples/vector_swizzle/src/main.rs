#![allow(dead_code)]

use macro_loop::macro_loop;

fn main() {
    let vector = Vec4 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
    };
    let swizzled = vector.yxwy();

    println!("{swizzled:?}");
}

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
