//mod native_types;
mod redis_type;

fn main() {
    let mut owned_string: String = String::from("hello ");
    let another_owned_string: String = String::from("world");
    
    owned_string.push_str(&another_owned_string);
    println!("{}", owned_string);
}
