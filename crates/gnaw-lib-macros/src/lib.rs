use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, FnArg, ItemFn};

#[proc_macro_attribute]
pub fn task(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    match task_inner(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn task_inner(item: ItemFn) -> Result<TokenStream, Error> {
    let name = &item.sig.ident;
    let task_ident = format_ident!("_gnaw_task_{}", name);

    let cloned = item.sig.clone();

    let types = item.sig.inputs.iter().map(|input| {
        match input {
            FnArg::Typed(pat) => Ok(pat.ty.clone()),
            FnArg::Receiver(_) => Err(Error::new_spanned(cloned.clone(), "Task functions do not support self receivers."))
        }
    }).collect::<Result<Vec<_>, Error>>()?;

    let variables = (0..types.len()).map(
        |i| { format_ident!("arg_{}", i) }
    ).collect::<Vec<_>>();

    Ok(quote!(
        #item

        #[unsafe(no_mangle)]
        pub extern "C" fn #task_ident(args: ::gnaw_lib::os::OsArgsData) {
            let args = args.into_os_strings();

            let mut iterator = args.into_iter();

            #(let Some(#variables) = <#types as ::gnaw_lib::arg::ConsoleArgs>::from_args(&mut iterator) else {
                println!("\x1b[1;31merror:\x1b[0m task is missing expected argument of type {}", stringify!(#types));
                return;
            };)*

            #name(#(#variables),*);
        }
    ).into())
}