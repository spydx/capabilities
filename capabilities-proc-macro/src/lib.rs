use proc_macro::TokenStream;
use syn::{AttributeArgs, parse_macro_input};
use darling::FromMeta;


#[derive(Debug, FromMeta)]
struct MacroAttributeArguments {

}



#[proc_macro_attribute]
pub fn capability(args: TokenStream, input: TokenStream) -> TokenStream {
    let _attr_args: AttributeArgs = parse_macro_input!(args);
    
    input
}

#[proc_macro_attribute]
pub fn database(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args: AttributeArgs = parse_macro_input!(args);

    let args = match MacroAttributeArguments::from_list(&attr_args) {
        Ok(val) => val, 
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }        
    };

    println!("{:?}", args);

    
    input
}
