#![feature(proc_macro_diagnostic)]
#![feature(let_else)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta, MetaNameValue};
use syn::{Lit, ItemStruct, Type, Ident};

const POOL_SQLITE: &str = "PoolSqlite";
const POOL_POSTGRES: &str = "PoolPostgres";
const WEB_SERVICE: &str = "WebService";
const CAP_PREFIX: &str = "Cap";

#[allow(dead_code)]
const FIELD_NAME: &str = "con";

/* 
    TODO: Missing naming of default field for the service struct.
    Now it is named "con" and should be user customizable for better readability
*/
#[proc_macro_attribute]
pub fn service(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(annotated_item);
    let input_args: AttributeArgs = parse_macro_input!(args);

    let service = input_args.first().cloned();

    //TODO: expand to more arguments, 
    // must be 1 or 2,and the first is a MetaList and the second one is NamedValue
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
                        POOL_SQLITE => Some(nm),
                        POOL_POSTGRES => Some(nm),
                        WEB_SERVICE => Some(nm),
                        _ => {
                            nm.span()
                                .unstable()
                                .error("Only \"PoolSqlite\", \"PoolPostgres\", or \"WebService\" allowed")
                                .emit();
                            None
                        }
                    };

                    t
                },
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
    let _service_field = parse_name_for_service_field(&input_args);
    //eprintln!("Service Field: {:?}", _service_field);
    let out = match service_token
        .path()
        .get_ident()
        .unwrap()
        .to_string()
        .as_str()
    {
        POOL_SQLITE => Some(impl_code_database(service_token, item)),
        POOL_POSTGRES=> Some(impl_code_database(service_token, item)),
        WEB_SERVICE => Some(impl_code_webservice(service_token, item)),
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

/*
 TODO: Missing IDENTIFIER #[id] for a field of the struct
 TODO: Missing matching data return for the operations, e.g Delete returns ()
    while CREATE returns #struct, ReadAll returns Vec<#struct> ...
*/

#[proc_macro_attribute]
pub fn capabilities(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(annotated_item);
    let attr_args: AttributeArgs = parse_macro_input!(args);

    let s = match item {
        Item::Struct(ref s) => Some(s.to_owned()),
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

    let item_struct = s.unwrap();

    let mut caps = vec![];
    for t in &attr_args {
        let m = match t {
            NestedMeta::Meta(m) => match m {
                Meta::Path(p) => Some(p),
                _ => None,
            },
            _ => None,
        };
        if m.is_some() {
            let val = m.unwrap();
            caps.push(val);
        }
    }
    let mut capidents = vec![];
    for cap in &caps {
        let capident = format_ident!("{}{}{}",CAP_PREFIX, cap.get_ident().unwrap(), item_struct.ident);
        capidents.push(capident);
    }

    let id_metavalue= parse_args_for_id_field(&attr_args);
        // this field needs to be dynamically assigned to different stuff.

    let struct_id = &item_struct.ident;
    let _id_type = parse_metavalue_for_type(&id_metavalue.clone().unwrap(), &item_struct);
    let generated_caps = generate_caps(&capidents, _id_type.clone(), &struct_id);

    quote! {
        #item_struct
        #( use ::capabilities::#caps;)*
        #generated_caps
    }.into()
    /*
    if id_metavalue.is_some() {
       
        let out = quote! {
            macro_rules! cap {
                ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                    #[async_trait]
                    pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
    
                    #[async_trait]
                    impl $name for $type {}
                };
            }
        
            #item_struct
        
            #( use ::capabilities::#caps;)*
            #(cap!( #capidents for CapService, composing { #caps<#_id_type>, #struct_id, CapServiceError}); )*
        };
        out.into()
    } else {
        let out = quote! {
            macro_rules! cap {
                ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                    #[async_trait]
                    pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
    
                    #[async_trait]
                    impl $name for $type {}
                };
            }
        
            #item_struct
            
            #( use ::capabilities::#caps;)*
            #(cap!( #capidents for CapService, composing { #caps<#struct_id>, #struct_id, CapServiceError}); )*
        };
        out.into()
    }*/
}
fn get_cap_macro() -> proc_macro2::TokenStream {
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

fn generate_caps(capabilities: &Vec<Ident>, id_type: Option<Type>, struct_name: &Ident) -> proc_macro2::TokenStream {
    
    let create = format_ident!("{}{}", "CapCreate", struct_name).to_string();
    let read = format_ident!("{}{}", "CapRead", struct_name).to_string();
    let update = format_ident!("{}{}", "CapUpdate", struct_name).to_string();
    let delete = format_ident!("{}{}", "CapDelete", struct_name).to_string();

    // create a vector and then flatten with repetition in quote!
    // use match to create the different out TokenStreams 
    let mut tokens = vec![];

    for cap in capabilities {

        let outtokens = 
        if cap.to_string().eq(&create) {
            Some(quote! {
                macro_rules! cap {
                    ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                        #[async_trait]
                        pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
            
                        #[async_trait]
                        impl $name for $type {}
                    };
                }
                cap!( #cap for CapService, composing { Create<#struct_name>, #struct_name, CapServiceError});
            })
        } else if cap.to_string().eq(&read) {
            if id_type.is_some() {
                Some(quote! {
                    macro_rules! cap {
                        ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                            #[async_trait]
                            pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
                
                            #[async_trait]
                            impl $name for $type {}
                        };
                    }
                    cap!( #cap for CapService, composing { Read<#id_type>, #struct_name, CapServiceError}); 
                })
            } else if id_type.is_none() {
               Some(quote! {
                    macro_rules! cap {
                        ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                            #[async_trait]
                            pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
                
                            #[async_trait]
                            impl $name for $type {}
                        };
                    }
                    cap!( #cap for CapService, composing { Read<#struct_name>, #struct_name, CapServiceError});
                })
            } else {
                None
            }
        } else if cap.to_string().eq(&update) {
            if id_type.is_some() {
                Some(quote! {
                    macro_rules! cap {
                        ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                            #[async_trait]
                            pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
                
                            #[async_trait]
                            impl $name for $type {}
                        };
                    }
                    cap!( #cap for CapService, composing { Update<#id_type>, #struct_name, CapServiceError}); 
                })
            } else if id_type.is_none() {
               Some(quote! {
                    macro_rules! cap {
                        ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                            #[async_trait]
                            pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
                
                            #[async_trait]
                            impl $name for $type {}
                        };
                    }
                    cap!( #cap for CapService, composing { Update<#struct_name>, #struct_name, CapServiceError});
                })
            } else {
                None
            }
        } else if cap.to_string().eq(&delete) {
            Some(quote! {
                macro_rules! cap {
                    ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
                        #[async_trait]
                        pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}
            
                        #[async_trait]
                        impl $name for $type {}
                    };
                }
                cap!( #cap for CapService, composing { Delete<#struct_name>, (), CapServiceError});
            })
        } else {
            None
        };
        if outtokens.is_some() {
            let t = outtokens.unwrap();
            tokens.push(t);
        }
    }

    quote! {
        #( #tokens )*
    }
}

fn parse_metavalue_for_type(id_metavalue: &MetaNameValue, item_struct: &ItemStruct) -> Option<Type> {
    
    let mut id_type = vec![];

    let id_field_name = match &id_metavalue.lit {
        Lit::Str(a) => Some(a.value()),
        _ => None,
    };
    if id_field_name.is_some() {
        let ident_fieldname = format_ident!("{}", &id_field_name.unwrap());
        for f in &item_struct.fields {
            let ident = f.ident.as_ref().unwrap();
            if ident.eq(&ident_fieldname) {
                id_type.push(f.to_owned().ty);
            }
        }
        let val = id_type.pop().unwrap();
        Some(val)
    } else {
        None
    }
}
fn parse_args_for_id_field(attr_args: &Vec<NestedMeta>) -> Option<MetaNameValue> {
    let mut id_vec = vec![];
    for i in attr_args {
        let m = match i {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(nv) => Some(nv),
                _ => None,
            },
            _ => None,
        };
        if m.is_some() {
            let val = m.unwrap();
            id_vec.push(val);
        }
        
    };
    if id_vec.is_empty() {
        None
    } else {
        let val = id_vec.pop().unwrap().to_owned();
        Some(val)
    }
}

fn parse_name_for_service_field(attr_args: &Vec<NestedMeta>) -> Option<MetaNameValue> {
    let mut id_vec = vec![];
    for i in attr_args {
        let m = match i {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(nv) => Some(nv),
                _ => None,
            },
            _ => None,
        };
        if m.is_some() {
            let val = m.unwrap();
            id_vec.push(val);
        }
    };
    if id_vec.is_empty() {
        None
    } else {
        let val = id_vec.pop().unwrap().to_owned();
        Some(val)
    }
}


#[proc_macro_attribute]
pub fn capability(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let mut attr_args: AttributeArgs = parse_macro_input!(args);
    let item: Item = parse_macro_input!(annotated_item);
   
    let s = match item {
        Item::Fn(ref s) => Some(s),
        _ => {
            item.span()
                .unstable()
                .error("We only support fn for now")
                .emit();
            None
        }
    };

    let arg_struct = attr_args.pop().unwrap();
    let arg_capability = attr_args.pop().unwrap();

    let arg_capability = match arg_capability {
        NestedMeta::Meta(m) => Some(m),
        _ => {
                arg_capability.span()
                .unstable()
                .error("Not a capability we support")
                .emit();
                None  
        }
    };

    let arg_struct = match arg_struct {
        NestedMeta::Meta(m) => Some(m),
        _ => {
                arg_capability.span()
                .unstable()
                .error("This should be a struct")
                .emit();
                None  
        }
    };

    let fn_signature = &s.unwrap().sig.ident;
    // can only hold one param
    let _fn_attrs = &s.unwrap().attrs;
    let fn_block = &s.unwrap().block;

    let item_struct = &arg_struct.unwrap().path().get_ident().unwrap().clone();
    let item_cap = &arg_capability.unwrap().path().get_ident().unwrap().clone();
    let capability = format_ident!("{}{}{}",CAP_PREFIX,item_cap, item_struct);
    
    let out = quote! {
        
        pub async fn #fn_signature<Service>(service: &Service, param: #item_struct ) -> Result<#item_struct, CapServiceError> 
        where
            Service: #capability,
        {
            service.perform(::capabilities::#item_cap { data: param }).await
        }

        #[async_trait]
        impl Capability<#item_cap<#item_struct>> for CapService {
            type Data = #item_struct;
            type Error = CapServiceError;

            async fn perform(&self, find_user: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                #fn_block
            }
        }
    };
    out.into()
}
