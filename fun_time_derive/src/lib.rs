use darling::FromMeta;
use quote::quote;
use syn::{parse_macro_input, ReturnType};

#[derive(FromMeta)]
enum When {
    Always,
    Debug,
}

impl Default for When {
    fn default() -> Self {
        Self::Always
    }
}

impl When {
    fn from_lit(literal: syn::LitStr) -> When {
        match literal.value().as_str() {
            "always" => Self::Always,
            "debug" => Self::Debug,
            // TODO: Compiler error
            unsupported => panic!(
                "Unsupported value for `when` attribute: {unsupported}. Use one of: always, debug"
            ),
        }
    }
}

#[derive(FromMeta)]
enum Reporting {
    Println,
    #[cfg(feature = "log")]
    Log,
}

impl Default for Reporting {
    fn default() -> Self {
        Self::Println
    }
}

impl Reporting {
    fn from_lit(literal: syn::LitStr) -> Reporting {
        match literal.value().as_str() {
            "println" => Self::Println,
            #[cfg(feature = "log")]
            "log" => Self::Log,
            // TODO: Compiler error
            unsupported => panic!("Unsupported value for `reporting` attribute: {unsupported}. Use one of: println, (only with log feature) log")
        }
    }
}

#[derive(FromMeta)]
struct FunTimeArgs {
    #[darling(default)]
    message: Option<String>,
    /// Determines when we should perform the timing.
    #[darling(default)]
    #[darling(map = "When::from_lit")]
    when: When,
    /// Determines whether the elapsed time should be returned or that we log it immediately
    /// to stdout.
    #[darling(default)]
    give_back: bool,
    #[darling(default)]
    #[darling(map = "Reporting::from_lit")]
    reporting: Reporting,
}

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
        return quote! { compile_error!("the `message` and `give_back` attributes are exclusive!") }
            .into();
    }

    let item_fn: syn::ItemFn = parse_macro_input!(item as syn::ItemFn);

    // Check if we should time the function
    match args.when {
        When::Debug if !cfg!(debug_assertions) => return quote! { #item_fn }.into(),
        _ => {} // No restrictions, go ahead!
    }

    // Deconstruct the signature
    let visibility = item_fn.vis;
    let syn::Signature {
        ident,
        generics,
        inputs,
        output,
        ..
    } = item_fn.sig;
    let where_clause = &generics.where_clause;

    // Contains the original logic of the function
    let block = item_fn.block;

    // Create wrapped function block
    let wrapped_block = quote! {
        let super_secret_variable_that_does_not_clash_start = std::time::Instant::now();

        let function_block = || { #block };
        let return_value = function_block();

        let elapsed = super_secret_variable_that_does_not_clash_start.elapsed();
    };

    // Depending on our `give_back` attibute we either return the elapsed time or not
    let tokens = if args.give_back {
        // Modify our output type to also return a std::time::Duration (our elapsed time)
        // In case of an empty return type we can simply return the std::time::Duration, otherwise
        // we have to wrap it in a tuple.
        let output_with_duration = match output {
            ReturnType::Default => syn::parse_str::<ReturnType>("-> std::time::Duration").unwrap(),
            ReturnType::Type(_, ty) => syn::parse_str::<ReturnType>(&format!(
                "-> ({}, std::time::Duration)",
                quote! { #ty }.to_string()
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
        let message = args.message.unwrap_or_else(String::new);

        let starting_statement = match args.reporting {
            Reporting::Println => quote! {
                println!("Starting: {}", #message);
            },
            #[cfg(feature = "log")]
            Reporting::Log => quote! {
                log::info!("Starting: {}", #message);
            },
        };

        let reporting_statement = match args.reporting {
            Reporting::Println => quote! {
                println!("{}: Done in {:.2?}", #message, elapsed);
            },
            #[cfg(feature = "log")]
            Reporting::Log => quote! {
                log::info!("{}: Done in {:.2?}", #message, elapsed);
            },
        };

        quote! {
            #visibility fn #ident #generics (#inputs) #output #where_clause {
                #starting_statement

                #wrapped_block

                #reporting_statement

                return_value
            }
        }
    };

    tokens.into()
}
