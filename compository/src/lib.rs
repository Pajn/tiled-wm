#[no_mangle]
pub extern fn print_hello_from_rust() {
    println!("Hello from Rust");
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}