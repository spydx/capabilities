#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;

#[allow(unused_imports)]
use darling::FromMeta;
#[allow(unused_imports)]
use quote::format_ident;
#[allow(unused_imports)]
use quote::quote;
use syn::__private::Span;
use syn::spanned::Spanned;

#[allow(unused_imports)]
use syn::{
    parse_macro_input, Attribute, AttributeArgs, Error, Field, Fields, Ident, Item, ItemStruct,
    Meta, NestedMeta, Result,
};

struct Items {
    pub item_struct: Option<ItemStruct>,
}

#[proc_macro_attribute]
pub fn svc(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(annotated_item);
    let input_args: AttributeArgs = parse_macro_input!(args);

    let service = input_args.first().cloned();

    if input_args.len() > 1 {
        service
            .span()
            .unstable()
            .error("Only supports one argument, SQLite or Request")
            .emit();
    }

    let service_type = match service.unwrap() {
        NestedMeta::Meta(nm) => {
            let allowed_type = match nm.clone() {
                Meta::Path(type_ident) => {
                    let t = match type_ident.get_ident().unwrap().to_string().as_str() {
                        "PoolSqlite" => Some(nm),
                        "WebService" => Some(nm),
                        _ => {
                            nm.span()
                                .unstable()
                                .error("Only \"PoolSqlite\" or \"WebService\" allowed")
                                .emit();
                            None
                        }
                    };

                    t
                }
                _ => {
                    let ident = nm.path().get_ident().unwrap().to_string();
                    nm.span()
                        .unstable()
                        .error(format!(
                            "Unknown type, there is no support for this type: {}",
                            ident
                        ))
                        .emit();
                    None
                }
            };
            Some(allowed_type)
        }
        _ => {
            input_args
                .first()
                .span()
                .unstable()
                .error("No literals allowed, only use SQlite or Reqwest")
                .emit();
            None
        }
    };
    let service_token = service_type.unwrap();

    let out = quote! {
        pub struct CapService {
            con: #service_token,
        }

        #[derive(Debug)]
        pub struct CapServiceError;


        impl CapService {
            pub async fn build(conf: String) -> Result<Self, crate::CapServiceError> {
                let con = Pool::connect(&conf)
                    .await
                    .expect("Failed to connect database");

                Ok ( Self { con: con })
            }
        }
        #item
    };
    out.into()
}

#[proc_macro_attribute]
pub fn service(_args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    /*
       1. Should be a struct
       2. Needs a build pattern for database and base url for client.
       3. Creates the trait for the others to inherit/implement.
    */
    let item: Item = parse_macro_input!(annotated_item);

    let s = match item {
        Item::Struct(ref s) => Some(s),
        _ => {
            item.span()
                .unstable()
                .error("Capability service can only annotate Structs")
                .emit();
            panic!("cannot continue compiling with this error")
        }
    };

    let field_name: Option<&Field> = match s.unwrap().fields {
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
    let mut error_str = ident.to_string();
    error_str.push_str("Error");
    let error_ident = syn::Ident::new(error_str.as_str(), Span::call_site());

    let db = field_name.unwrap().ident.as_ref().unwrap();

    let out = quote! {
        #s
        #[derive(Debug)]
        pub struct #error_ident;

        impl #ident {
            pub async fn build(conf: String) -> Result<Self, #error_ident> {
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
