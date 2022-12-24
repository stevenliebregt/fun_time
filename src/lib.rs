pub use fun_time_derive::*;

#[cfg(test)]
mod tests {
    use fun_time_derive::fun_time;
    use simple_logger::SimpleLogger;
    use std::fmt::Debug;
    use std::time::Duration;

    #[fun_time(give_back)]
    fn dummy_test_function_that_sleeps<'a, T>(borrowed_thing: &'a T) -> &'a T
    where
        T: Debug + Sized,
    {
        std::thread::sleep(Duration::from_millis(42));

        println!("the borrowed_thing is = {borrowed_thing:?}");

        borrowed_thing
    }

    #[fun_time(when = "debug", message = "having fun with log", reporting = "log")]
    fn have_fun(_first: String, _second: String) {
        std::thread::sleep(Duration::from_millis(69));
    }

    #[test]
    fn it_works() {
        let (borrowed_thing, elapsed_time) = dummy_test_function_that_sleeps(&"Hello, there!");

        assert_eq!(&"Hello, there!", borrowed_thing);

        // Bit of wiggle room
        assert!(
            elapsed_time > Duration::from_millis(40) && elapsed_time < Duration::from_millis(44)
        );

        SimpleLogger::new().init().unwrap_or(());

        have_fun("Alice".to_string(), "Bob".to_string());
    }
}
