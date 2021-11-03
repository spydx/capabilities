#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use syn::{parse_macro_input, AttributeArgs, Item, ItemStruct, Meta, NestedMeta};

struct Items {
    pub item_struct: Option<ItemStruct>,
}

#[proc_macro_attribute]
pub fn service(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(annotated_item);
    let input_args: AttributeArgs = parse_macro_input!(args);

    let service = input_args.first().cloned();

    if input_args.len() > 1 {
        service
            .span()
            .unstable()
            .error("Only supports one argument")
            .emit();
    }

    if service.is_none() {
        service
            .span()
            .unstable()
            .error("Unsupported literal, has to be a type from Capabilities lib")
            .emit();
    }

    let service_type = match service.unwrap() {
        NestedMeta::Meta(nm) => {
            let allowed_type = match nm.clone() {
                Meta::Path(type_ident) => {
                    let t = match type_ident.get_ident().unwrap().to_string().as_str() {
                        "PoolSqlite" => Some(nm),
                        "PoolPostgres" => Some(nm),
                        "WebService" => Some(nm),
                        _ => {
                            nm.span()
                                .unstable()
                                .error("Only \"PoolSqlite\", \"PoolPostgres\", or \"WebService\" allowed")
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
                .error("No literals allowed")
                .emit();
            None
        }
    };
    let service_token = service_type.unwrap().unwrap();

    let out = match service_token
        .path()
        .get_ident()
        .unwrap()
        .to_string()
        .as_str()
    {
        "PoolSqlite" => Some(impl_code_database(service_token, item)),
        "PoolPostgres" => Some(impl_code_database(service_token, item)),
        "WebService" => Some(impl_code_webservice(service_token, item)),
        _ => {
            service_token
                .span()
                .unstable()
                .error("Unknown error")
                .emit();
            None
        }
    };
    if out.is_none() {}
    out.unwrap()
}

fn impl_code_database(service_token: Meta, item: Item) -> TokenStream {
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

fn impl_code_webservice(service_token: Meta, item: Item) -> TokenStream {
    let out = quote! {
        pub struct CapService {
            con: #service_token,
        }

        #[derive(Debug)]
        pub struct CapServiceError;

        impl CapService {
            pub async fn build() -> Result<Self, crate::CapServiceError> {
                let con = Client::new();

                Ok(Self { con: con })
            }
        }
        #item
    };

    out.into()
}

#[proc_macro_attribute]
pub fn capabilities(_args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(annotated_item);
    let out = quote! {
        #item
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
