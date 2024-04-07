pub mod scanners;


fn main() {
    let z = scanners::connections::conn_info();
    println!("Connection info: {:?}", z);
}
