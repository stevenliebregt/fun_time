use darling::FromMeta;
use quote::quote;
use syn::{parse_macro_input, ReturnType};

macro_rules! make_darling_error {
    ($($arg:tt)*) => {
        Err(darling::Error::custom(format!($($arg)*)))
    };
}

macro_rules! make_compile_error {
    ($($arg:tt)*) => {
        quote! { compile_error!($($arg)*) }.into()
    };
}

/// Determines when to collect execution time information.
#[derive(FromMeta)]
enum When {
    /// Always collect timing information.
    Always,
    /// Only collect timing information if `cfg!(debug_assertions)` evaluates to `true`.
    Debug,
}

/// By default we always collect timing information.
impl Default for When {
    fn default() -> Self {
        Self::Always
    }
}

impl When {
    /// Parse the [`When`] argument from a given string literal.
    fn from_lit(literal: syn::LitStr) -> Result<Self, darling::Error> {
        match literal.value().as_str() {
            "always" => Ok(Self::Always),
            "debug" => Ok(Self::Debug),
            unsupported => make_darling_error!(
                "Unsupported value for `when` attribute: {unsupported}. Use one of: always, debug"
            ),
        }
    }
}

/// Determines how to report the captured execution time information.
///
/// It will print both a start and done message.
/// The format of the start message is: "Starting: YOUR_MESSAGE_HERE"
/// The format of the done message is: "YOUR_MESSAGE_HERE: Done in ELAPSED_TIME"
///
/// The `ELAPSED_TIME` is the debug format of [`std::time::Duration`].
#[derive(FromMeta)]
enum Reporting {
    /// Use a simple `println!` statement to print the information to the `stdout`.
    Println,
    /// Use the [log](https://crates.io/crates/log) crate to print the information using the
    /// provided `info!` macro.
    #[cfg(feature = "log")]
    Log,
}

/// By default we use the simple `println!` to write the reporting info to the `stdout`.
impl Default for Reporting {
    #[cfg(not(feature = "log"))]
    fn default() -> Self {
        Self::Println
    }

    #[cfg(feature = "log")]
    fn default() -> Self {
        Self::Log
    }
}

impl Reporting {
    /// Parse the [`Reporting`] argument from a given string literal.
    fn from_lit(literal: syn::LitStr) -> Result<Self, darling::Error> {
        match literal.value().as_str() {
            "println" => Ok(Self::Println),
            #[cfg(feature = "log")]
            "log" => Ok(Self::Log),
            unsupported => make_darling_error!("Unsupported value for `reporting` attribute: {unsupported}. Use one of: println, (only with log feature) log")
        }
    }
}

#[cfg(feature = "log")]
mod log_level {
    use super::*;
    use std::str::FromStr;

    pub struct Level(pub log::Level);

    impl FromMeta for Level {}

    impl Default for Level {
        fn default() -> Self {
            Self(log::Level::Info)
        }
    }

    impl Level {
        pub fn from_lit(literal: syn::LitStr) -> Result<Self, darling::Error> {
            Ok(Self(log::Level::from_str(&literal.value()).map_err(|_| {
                darling::Error::custom(format!(
                    "Unsupported value for `level` attribute: {unsupported}. Use one of: trace, debug, info, warn, error",
                    unsupported = literal.value()
                ))
            })?))
        }
    }
}

#[derive(FromMeta)]
struct FunTimeArgs {
    #[darling(default)]
    message: Option<String>,
    /// Determines when we should perform the timing.
    #[darling(default)]
    #[darling(and_then = "When::from_lit")]
    when: When,
    /// Determines whether the elapsed time should be returned or that we log it immediately
    /// to stdout.
    #[darling(default)]
    give_back: bool,
    #[darling(default)]
    #[darling(and_then = "Reporting::from_lit")]
    reporting: Reporting,

    #[cfg(feature = "log")]
    #[darling(default)]
    #[darling(and_then = "log_level::Level::from_lit")]
    level: log_level::Level,
}

/// Measure the execution times of the function under the attribute.
///
/// It does this by wrapping the function in a new block surrounded by a [`std::time::Instant::now()`]
/// call and a [`std::time::Instant::elapsed()`] call. It then either logs the duration directly or
/// returns it with the original functions return value, depending on how you configured this
/// attribute.
///
/// # Attributes
///
/// ## when
///
/// The `when` attribute can be used to configure when the timing information is collected. For
/// example, with `"always"` the timing information is always collected, but with `"debug"` the
/// timing information is only collected if the `cfg!(debug_assertions)` statement evaluates to
/// `true`.
///
/// ## give_back
///
/// The `give_back` attribute can be used to switch the macro from printing mode to returning the
/// captured elapsed time together with the original return value of the function. It will modify
/// the original return value to be a tuple, where the first value is the original return value
/// and the second value is the elapsed time as a [`std::time::Duration`] struct.
///
/// ## message
///
/// The `message` attribute allows you to set a message that will be displayed in the case you
/// chose to let the macro report the elapsed time directly. This message will be shown both in
/// the start and done messages.
///
/// ## reporting
///
/// The `reporting` attribute determines how the message and elapsed time will be displayed
/// directly when you have chosen not to let the macro return the elapsed time to you. By default
/// it uses a simple `println!` statement, but with the optional `log` feature it will use the
/// [log](https://crates.io/crates/log) crate to log it using the `info!` macro.
///
/// # Example
///
/// ```
/// // Replace this in your code with `use fun_time::fun_time;`
/// use fun_time_derive::fun_time;
///
/// #[fun_time(give_back)]
/// fn function_with_heavy_calculations(some_data: Vec<i32>) -> bool {
///     // Big brain calculations...
///     true
/// }
///
/// fn main() {
///     let my_data = vec![1, 2, 3];
///
///     // Run the function and receive timing information
///     let (my_original_return_value, how_long_did_it_take) = function_with_heavy_calculations(my_data);
/// }
/// ```
#[proc_macro_attribute]
pub fn fun_time(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let raw_args: syn::AttributeArgs = parse_macro_input!(args as syn::AttributeArgs);
    let args: FunTimeArgs = match FunTimeArgs::from_list(&raw_args) {
        Ok(args) => args,
        Err(error) => return error.write_errors().into(),
    };

    if args.message.is_some() && args.give_back {
        return make_compile_error!(
            "the `message` and `give_back` attributes can not be used together!"
        );
    }

    let item_fn: syn::ItemFn = parse_macro_input!(item as syn::ItemFn);

    // Check if we should time the function
    match args.when {
        When::Debug if args.give_back => return make_compile_error!("the `give_back` and `when` attribute with `\"debug\"` can not be used together! It would result in different return types"),
        When::Debug if !cfg!(debug_assertions) => return quote! { #item_fn }.into(),
        _ => {} // No restrictions, go ahead!
    }

    let visibility = item_fn.vis;
    let signature = item_fn.sig.clone();

    // Store original return type to support functions that return for example: `Box<dyn Trait>`
    let return_type = match item_fn.sig.output {
        syn::ReturnType::Type(_, ty) => quote!{ #ty },
        syn::ReturnType::Default => quote!{ () },
    };

    // Contains the original logic of the function
    let block = item_fn.block;

    // Create wrapped function block
    let wrapped_block = quote! {
        let super_secret_variable_that_does_not_clash_start = std::time::Instant::now();

        // Immediately invoked closure so a `return` statement in the original function does not
        // break the logging. This also works with self-mutating structs.
        let return_value: #return_type = (|| { #block })();

        let elapsed = super_secret_variable_that_does_not_clash_start.elapsed();
    };

    // Create tokens for the `log` call if it is enabled
    #[cfg(feature = "log")]
    let log_tokens = match args.level.0 {
        log::Level::Error => quote! { log::error! },
        log::Level::Warn => quote! { log::warn! },
        log::Level::Info => quote! { log::info! },
        log::Level::Debug => quote! { log::debug! },
        log::Level::Trace => quote! { log::trace! },
    };

    // Depending on our `give_back` attibute we either return the elapsed time or not
    let tokens = if args.give_back {
        // Deconstruct the signature because we need to edit the return type
        let syn::Signature {
            ident,
            generics,
            inputs,
            output,
            ..
        } = signature;
        let where_clause = &generics.where_clause;

        // Modify our output type to also return a std::time::Duration (our elapsed time)
        // In case of an empty return type we can simply return the std::time::Duration, otherwise
        // we have to wrap it in a tuple.
        let output_with_duration = match output {
            ReturnType::Default => syn::parse_str::<ReturnType>("-> std::time::Duration").unwrap(),
            ReturnType::Type(_, ty) => syn::parse_str::<ReturnType>(&format!(
                "-> ({}, std::time::Duration)",
                quote! { #ty }
            ))
                .unwrap(),
        };

        quote! {
            #visibility fn #ident #generics (#inputs) #output_with_duration #where_clause {
                #wrapped_block

                (return_value, elapsed)
            }
        }
    } else {
        let message = args.message.unwrap_or_default();

        // Store the message at the top of the function because if the function were to take
        // ownership of the argument it would be gone by the time we want to print the done message.
        let message_statement = quote! {
            let super_secret_variable_that_does_not_clash_message = format!(#message);
        };

        let starting_statement = match args.reporting {
            Reporting::Println => quote! {
                println!("{}", super_secret_variable_that_does_not_clash_message);
            },
            #[cfg(feature = "log")]
            Reporting::Log => quote! {
                #log_tokens("{}", super_secret_variable_that_does_not_clash_message);
            },
        };

        let reporting_statement = match args.reporting {
            Reporting::Println => quote! {
                println!("{}: Done in {:.2?}", super_secret_variable_that_does_not_clash_message, elapsed);
            },
            #[cfg(feature = "log")]
            Reporting::Log => quote! {
                #log_tokens("{}: Done in {:.2?}", super_secret_variable_that_does_not_clash_message, elapsed);
            },
        };

        quote! {
            #visibility #signature {
                #message_statement
                #starting_statement

                #wrapped_block

                #reporting_statement

                return_value
            }
        }
    };

    tokens.into()
}
