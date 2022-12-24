use std::time::Duration;
use fun_time::fun_time;

#[fun_time(give_back)]
fn some_heavy_calculations(some_data: &[i32]) -> String {
    std::thread::sleep(Duration::from_millis(69));

    // This has nothing to do with math :)
    format!("I did my math homework, some_data is {} items long", some_data.len())
}

fn main() {
    println!("Hello, world!");

    let my_data = vec![1, 1, 2, 3, 5, 8, 13];

    let (the_string, elapsed_time) = some_heavy_calculations(&my_data);

    println!("{the_string}");
    println!("Getting that value took: {elapsed_time:.2?}");
}
