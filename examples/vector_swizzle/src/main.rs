#![allow(dead_code)]

use macro_loop::macro_loop;

fn main() {
    let vector = Vec4 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
    };
    let mut swizzled = vector.yxwy();

    swizzled.set_xwyz(swizzled);

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

        // Get
        @for X in @components, Y in @components, Z in @components, W in @components {
            pub fn @[@X @Y @Z @W](self) -> Vec4 {
                Vec4 { x: self.@X, y: self.@Y, z: self.@Z, w: self.@W }
            }
        }

        // Set
        // We want to skip all conflicting combinations (set_xxyz...)
        @for X in @components, Y in @components, Z in @components, W in @components {
            @if
                @X != @Y && @X != @Z && @X != @W
                && @Y != @Z && @Y != @W
                && @Z != @W
            {
                pub fn @[set_ @X @Y @Z @W](&mut self, value: Vec4) {
                    self.@X = value.x;
                    self.@Y = value.y;
                    self.@Z = value.z;
                    self.@W = value.w;
                }
            }
        }
    }
}
