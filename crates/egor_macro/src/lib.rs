use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    TokenStream::from(quote::quote! {
        #[cfg(target_os = "android")]
        #[unsafe(no_mangle)]
        fn android_main(app: ::egor::app::AndroidApp) {
            let _ = ::egor::app::ANDROID_APP.set(app);
            main();
        }

        #input
    })
}
