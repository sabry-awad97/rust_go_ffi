mod bindings {
    #![allow(unused)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;

fn main() {
    unsafe {
        println!("Calling Go function from Rust...");
        GoFunction();

        let result = AddNumbers(5, 10);
        println!("5 + 10 = {}", result);
    }
}
