pub use fun_time_derive::*;

#[cfg(test)]
mod tests {
    use fun_time_derive::fun_time;
    use simple_logger::SimpleLogger;
    use std::fmt::{Debug, Formatter};
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

    struct Parameter {
        name: String,
        value: i32,
    }

    impl Debug for Parameter {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if f.alternate() {
                write!(f, "name = {} and value * 2 = {}", self.name, self.value * 2)
            } else {
                write!(f, "name = {} and value = {}", self.name, self.value)
            }
        }
    }

    #[fun_time(message = "having fun with parameters: {first:#?} and {second}")]
    fn have_fun_with_parameters(first: Parameter, second: i32) {
        // Let's take ownership of the first parameter
        let _first = first;
        std::thread::sleep(Duration::from_millis(42));
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

    #[test]
    fn it_works_with_parameters() {
        SimpleLogger::new().init().unwrap_or(());

        have_fun_with_parameters(
            Parameter {
                name: "Alice".to_string(),
                value: 1234,
            },
            1234,
        );
    }
}
