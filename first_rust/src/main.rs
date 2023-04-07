
// use std::fs;
// main 函数现在返回一个 Result
fn main() -> Result<(), Box<dyn std::error::Error>> {

    for arg in std::env::args() {
        println!("{}", arg);
    }

    Ok(())
}