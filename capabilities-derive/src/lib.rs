#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;

#[allow(unused_imports)]
use darling::FromMeta;
#[allow(unused_imports)]
use quote::format_ident;
#[allow(unused_imports)]
use quote::quote;
use syn::spanned::Spanned;
#[allow(unused_imports)]
use syn::{parse, parse_macro_input, AttributeArgs, DeriveInput};
use syn::{Field, Fields, Item, ItemStruct};

struct Items {
    pub item_struct: Option<ItemStruct>,
}

#[proc_macro_attribute]
pub fn service(_args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    /*
       1. Should be a struct
       2. Needs a build pattern for database and base url for client.
       3. creates the trait for the others to inherit/implement.
    */
    let item: Item = parse_macro_input!(annotated_item);

    let s = match item {
        Item::Struct(ref s) => Some(s),
        _ => {
            item.span()
                .unstable()
                .error("Capability service can only annotate Structs")
                .emit();
            None
        }
    };

    let fieldname: Option<&Field> = match s.unwrap().fields {
        Fields::Named(ref f) => f.named.first().to_owned(),
        Fields::Unnamed(_) => {
            eprintln!("Fields of struct must be named");
            None
        }
        Fields::Unit => {
            eprintln!("Cannot be a unit filed");
            None
        }
    };


    let ident = s.unwrap().ident.to_owned();
    let db = fieldname.unwrap().ident.as_ref().unwrap();

    let out = quote! {
        #s
        #[derive(Debug)]
        struct DatabaseError;

        impl #ident {
            pub async fn build(conf: String) -> Result<Self, DatabaseError> {
                Ok ( Self { #db: conf })
            }
        }
    };
    out.into()
}

#[proc_macro_attribute]
pub fn capability(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    /*
       1. Implements for the struct and functions
       2. for struct, it creates the cap! traits
       3. for fn it implements the trait.
    */
    let _attr_args: AttributeArgs = parse_macro_input!(args);
    let item: Item = parse_macro_input!(annotated_item);

    let s = match item {
        Item::Struct(ref s) => {
            eprintln!("{}", s.ident);
            Items {
                item_struct: Some(s.to_owned()),
            }
        }
        _ => {
            item.span()
                .unstable()
                .error("We only support structs for now")
                .emit();
            Items { item_struct: None }
        }
    };
    let ident = s.item_struct.unwrap().ident;

    eprintln!("{:?}", ident);

    let out = quote! {
        #item
        use async_trait::async_trait;
        pub struct User;
        pub struct DatabaseError;
        pub struct Read<T>(T);
        #[async_trait]
        pub trait Capability<Operation> {
            type Data;
            type Error;
            async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
        }

        #[async_trait]
        impl Capability<Read<String>> for #ident {
            type Data = User;
            type Error = DatabaseError;

            async fn perform(&self, find_user: Read<String>) -> Result<Self::Data, Self::Error> {
                Ok(User)
            }
        }


    };
    out.into()
}
