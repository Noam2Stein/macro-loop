use macro_loop::macro_loop;
use paste::paste;

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
    macro_loop! { paste! {
        @for X in 0..4 {
            pub fn xy(self) -> X {
                Vec2 { x: self.x, y: self.y }
            }
        }
    } }
}
