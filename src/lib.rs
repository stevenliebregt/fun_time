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
    fn have_fun(_first: String, _second: String) {}

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

    #[fun_time(
        message = "having fun with parameters: {first:#?} and {second}",
        level = "debug",
        reporting = "log"
    )]
    fn have_fun_with_parameters(first: Parameter, second: i32) {
        // Let's take ownership of the first parameter
        let _first = first;
    }

    #[test]
    fn it_works() {
        let (borrowed_thing, _elapsed_time) = dummy_test_function_that_sleeps(&"Hello, there!");

        assert_eq!(&"Hello, there!", borrowed_thing);

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

    #[test]
    fn works_with_struct_member_that_modifies() {
        struct Thing {
            value: i32,
        }

        impl Thing {
            pub fn new(value: i32) -> Self {
                Self { value }
            }

            #[fun_time(message = "modify while mutating self", reporting = "println")]
            pub fn modify_timed(&mut self, new_value: i32) -> i32 {
                let old_value = self.value;
                self.value = new_value;
                old_value
            }
        }

        let mut thing = Thing::new(1);
        let _old_value = thing.modify_timed(1337);
    }

    #[test]
    fn works_with_traits() {
        trait MyTrait {
            fn speak(&self) -> &'static str;
        }

        struct A {}
        impl MyTrait for A {
            fn speak(&self) -> &'static str {
                "I am: A"
            }
        }

        struct B {}
        impl MyTrait for B {
            fn speak(&self) -> &'static str {
                "I am: B"
            }
        }

        #[derive(Debug)]
        enum MyEnum {
            A,
            B,
        }

        impl MyEnum {
            #[fun_time(message = "Getting trait item for: {self:?}")]
            fn get_trait_item(&self) -> Box<dyn MyTrait> {
                match self {
                    Self::A => Box::new(A {}),
                    Self::B => Box::new(B {}),
                }
            }
        }

        let enum_a = MyEnum::A;
        let _ = enum_a.get_trait_item().speak();

        let enum_b = MyEnum::B;
        let _ = enum_b.get_trait_item().speak();
    }
}
