use macro_loop::macro_loop;

macro_loop! {
    fn main() {
        println!(@[concat _ strings => str]);

        // outputs:
        // println!("concat_strings");
    }
}
