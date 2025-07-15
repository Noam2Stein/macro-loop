use macro_loop::macro_loop;

fn main() {
    macro_loop! {
        @for [KEY, VALUE] in [["one", 1], ["two", 2], ["three", 3]] {
            println!("{} => {}", @KEY, @VALUE);
        }

        @let [A, B] = ['a', 'b'];

        println!("{}, {}", @A, @B);
    }
}
