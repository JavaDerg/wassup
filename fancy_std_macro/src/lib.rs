use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn async_main(
    _args: TokenStream,
    input: TokenStream
) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);
    let body = input.block;

    let sig = input.sig;
    if sig.ident.to_string() != "main"
        || sig.asyncness.is_none()
        || !matches!(sig.output, syn::ReturnType::Default)
        || !sig.inputs.is_empty()
        || sig.variadic.is_some()
        || sig.unsafety.is_some()
        || !sig.generics.params.is_empty()
        || sig.constness.is_some() {
        quote! {
            compiler_error!("the function used with `#[fancy_std::main]` must be `async fn main() -> ()`");
        }
    } else {
        quote! {
            #[no_mangle]
            pub extern "C" fn _start() {
                let _ = ::fancy_std::spawn(async {
                    ::fancy_std::startup_runtime();
                    async {
                        #body
                    }.await;
                    ::fancy_std::shutdown_runtime();
                });
            }
        }
    }.into()
}
