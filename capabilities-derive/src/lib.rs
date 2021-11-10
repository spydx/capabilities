#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta};

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
        use async_trait::async_trait;
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
        #[async_trait]
        pub trait Capability<Operation> {
            type Data;
            type Error;
            async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
        }
        #item
    };
    out.into()
}

fn impl_code_webservice(service_token: Meta, item: Item) -> TokenStream {
    let out = quote! {
        use async_trait::async_trait;
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
        #[async_trait]
        pub trait Capability<Operation> {
            type Data;
            type Error;
            async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
        }
        #item
    };

    out.into()
}

#[allow(dead_code)]
fn gen_trait() -> TokenStream2 {
    quote! {
        #[async_trait]
        pub trait Capability<Read<String>, Data = Orders, Error = CapServiceError> for CapService {
            
        }
    }
}


#[proc_macro_attribute]
pub fn capabilities(args: TokenStream, annotated_item: TokenStream) -> TokenStream {

    let item: Item = parse_macro_input!(annotated_item);

    let attr_args: AttributeArgs = parse_macro_input!(args);


    let s = match item {
        Item::Struct(ref s) =>
            Some(s.to_owned()),
        _ => {
            item.span()
                .unstable()
                .error("We only support structs")
                .emit();
                None
        }
    };
    if s.is_none() {
        panic!("Cannot continue");
    }

    let ident_struct = s.unwrap();
    
    let mut caps = vec![];

    for t in attr_args {
        let m = match t {
            NestedMeta::Meta(m) => {
                match m {
                    Meta::Path(ident) => Some(ident),
                    _ => None,
                }
            },
            _ => None,
        };
        if m.is_some() {
            let val = m.unwrap();
            caps.push(val);
        }
    }
    let canread = format_ident!("Can{}{}", "Read", ident_struct.ident);

    let out = quote! {
        use ::capabilities::Read;

        #ident_struct
        
        macro_rules! cap {
            ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                #[async_trait]
                pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
        
                #[async_trait]
                impl $name for $type {}
            };
        }
        cap! (#canread for CapService, composing { Read<String>, #ident_struct.ident, CapServiceError});
    };
    out.into()
}


#[proc_macro_attribute]
pub fn capability(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    /*
       1. Implements for the fn
       3. for fn it implements the trait.
    */
    let _attr_args: AttributeArgs = parse_macro_input!(args);
    let item: Item = parse_macro_input!(annotated_item);

    let s = match item {
        Item::Fn(ref s) => {
            Some(s)
        }
        _ => {
            item.span()
                .unstable()
                .error("We only support fn for now")
                .emit();
                None
        }
    };
    let ident = &s.unwrap().sig;

    eprintln!("{:?}", ident);

    let out = quote! {
        #item
        use async_trait::async_trait;
        pub struct User;

        pub struct Read<T>(T);

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
