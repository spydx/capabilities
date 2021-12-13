use quote::{quote, format_ident};
use syn::Ident;
use proc_macro2::{TokenStream};

pub fn get_id_identifier() -> Ident {
    format_ident!("{}", "id")
} 

pub fn get_name_identifier() -> Ident {
    format_ident!("{}", "name")
}

pub fn get_cap_macro() -> TokenStream {
    quote! {
        macro_rules! cap {
        ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
            #[async_trait]
            pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}

            #[async_trait]
            impl $name for $type {}
        };
    }}
}