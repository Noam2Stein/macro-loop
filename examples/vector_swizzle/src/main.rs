use macro_loop::macro_loop;

fn main() {}

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
        @for X in [x, y] {
            @let swizzle = @X + @X;
            pub fn @swizzle(self) -> f32 {
                self.@X
            }
        }
    }
}
