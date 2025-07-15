use macro_loop::macro_loop;

fn main() {
    let vector = Vec2 { x: 1.0, y: 2.0 };
    let swizzled = vector.yx();

    println!("{swizzled:?}");
}

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec2 {
    macro_loop! {
        @for X in [x, y] { @for Y in [x, y] {
            @let swizzle = @X + @Y;
            pub fn @swizzle(self) -> Vec2 {
                Vec2 { x: self.@X, y: self.@Y }
            }
        } }
    }
}
