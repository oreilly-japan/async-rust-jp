
fn perform_operation_with_callback<F>(callback: F)
where
    F: Fn(i32),
{
    let result = 42;
    callback(result);
}

fn main() {
    let my_callback = |result: i32| {
        println!("The result is: {}", result);
    };
    perform_operation_with_callback(my_callback);
}
 