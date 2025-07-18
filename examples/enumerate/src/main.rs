#![allow(dead_code)]

use macro_loop::macro_loop;

fn main() {
    let vector = Vec3 {
        array: [1.0, 2.0, 3.0],
    };

    println!("{}", vector.y());
}

struct Vec3 {
    array: [f32; 3],
}

macro_loop! {
    impl Vec3 {
        @for [idx, c] in [x, y, z].enumerate() {
            fn @c(self) -> f32 {
                self.array[@idx]
            }
        }

        @for [idx, c] in [r, g, b].enumerate() {
            fn @c(self) -> f32 {
                self.array[@idx]
            }
        }

        @for start in 0..3, end in @start..3 {
            @let components = [x, y, z][@start..=@end];

            fn @[@components _ref](&self) -> &[f32] {
                &self.array[@start..=@end]
            }
        }
    }
}
