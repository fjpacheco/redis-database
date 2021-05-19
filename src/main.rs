mod native_types;

fn main() {
    let mut owned_string: String = String::from("hello ");
    let another_owned_string: String = String::from("world");
    
    owned_string.push_str(&another_owned_string);
    println!("{}", owned_string);
}
