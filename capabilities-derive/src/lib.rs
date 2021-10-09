use proc_macro::TokenStream;
use quote::format_ident;
use syn::{AttributeArgs, parse_macro_input};
use darling::FromMeta;


#[derive(Debug, FromMeta)]
struct MacroAttributeArguments {
    #[darling(default)]
    enumerate: TOption<String>,
}


#[derive(Debug)]
enum TOption<T> {
    Some(T),
    Default, 
    None,
}

impl<T> Default for TOption<T> {
    fn default() -> Self {
        Self::None
    }
}

impl FromMeta for TOption<String> {
    fn from_string(value: &str) -> darling::Result<Self> {
        if value.is_empty() {
            return Ok(Self::Default);
        }
        Ok(Self::Some(value.to_string()))
    }
}


#[proc_macro_attribute]
pub fn capability(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args: AttributeArgs = parse_macro_input!(args);

    let args = match MacroAttributeArguments::from_list(&attr_args) {
        Ok(val) => val, 
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }        
    };

    let cap_idents = match args.enumerate {
        TOption::Some(string) => Some(format_ident!("{}", string)),
        TOption::Default =>  Some(format_ident!("E {}", "default")),
        TOption::None => None,
    };


    println!("{:?}", cap_idents);
    
    input
}

#[proc_macro_attribute]
pub fn database(_args: TokenStream, input: TokenStream) -> TokenStream {
    
    input
}