#![feature(proc_macro_diagnostic)]
mod helpers;

use helpers::{
    generate_caps, impl_code_database, impl_code_webservice, parse_field_args_for_id,
    parse_metavalue_for_type, parse_service_field_for_name,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta};

const POOL_SQLITE: &str = "PoolSqlite";
const POOL_POSTGRES: &str = "PoolPostgres";
const WEB_SERVICE: &str = "WebService";
const CAP_PREFIX: &str = "Cap";

#[proc_macro_attribute]
pub fn service(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let item: Item = parse_macro_input!(annotated_item);
    let input_args: AttributeArgs = parse_macro_input!(args);

    let service = input_args.first().cloned();

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
    let service_field = parse_service_field_for_name(&input_args);

    let out = match service_token
        .path()
        .get_ident()
        .unwrap()
        .to_string()
        .as_str()
    {
        POOL_SQLITE => Some(impl_code_database(service_token, item, service_field)),
        POOL_POSTGRES => Some(impl_code_database(service_token, item, service_field)),
        WEB_SERVICE => Some(impl_code_webservice(service_token, item, service_field)),
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
        let capident = format_ident!(
            "{}{}{}",
            CAP_PREFIX,
            cap.get_ident().unwrap(),
            item_struct.ident
        );
        capidents.push(capident);
    }

    let id_metavalue = parse_field_args_for_id(&attr_args);
    // this field needs to be dynamically assigned to different stuff.

    let struct_id = &item_struct.ident;
    let _id_type = parse_metavalue_for_type(&id_metavalue.clone(), &item_struct);

    let generated_caps = generate_caps(&capidents, _id_type.clone(), &struct_id);

    quote! {
        #item_struct
        #( use ::capabilities::#caps;)*
        #generated_caps
    }
    .into()
}

#[proc_macro_attribute]
pub fn capability(args: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let mut attr_args: AttributeArgs = parse_macro_input!(args);
    let item: Item = parse_macro_input!(annotated_item);

    if attr_args.is_empty() {
        item.span()
            .unstable()
            .error("Missing arguments Capability and Struct")
            .emit();
    }


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

    if s.is_none() { 
        s.span().unstable().error("Missing function").emit();
    }

    let arg_path = attr_args.pop();
    let arg_struct = attr_args.pop();
    let arg_capability = attr_args.pop();
    
    
    let arg_path = if arg_path.is_some() { 
        match arg_path.unwrap() {
            NestedMeta::Meta(p) => match p {
                Meta::NameValue(nv) => {
                    let field_name = format_ident!("{}", "id");
                    if nv.path.get_ident().unwrap().eq(&field_name) {
                        Some(nv)
                    } else {
                        None
                    }
                },
                _ => None
            },
            _ => None
        }
    } else {
        None
    };

    let arg_capability = if arg_capability.is_some() {
        match arg_capability.as_ref().unwrap() {
            NestedMeta::Meta(m) => Some(m),
            _ => {
                arg_capability
                    .span()
                    .unstable()
                    .error("Not a capability we support")
                    .emit();
                None
            }
        }
    } else {
        arg_capability.span().unstable().error("Missing capability / struct / wrong order").emit();
        None
    };

    let arg_struct = if arg_struct.is_some() {
        match arg_struct.unwrap() {
            NestedMeta::Meta(m) => Some(m),
            _ => {
                arg_capability
                    .span()
                    .unstable()
                    .error("This should be a struct")
                    .emit();
                None
            }
        }
    } else {
        arg_capability.span().unstable().error("Missing capability / struct / wrong order").emit();
        None
    };

    eprintln!("{:?}", arg_path);
    eprintln!("{:?}", arg_struct);
    eprintln!("{:?}", arg_capability);
   
    let fn_signature = &s.unwrap().sig.ident;
    // can only hold one param
    let _fn_attrs = &s.unwrap().attrs;
    let fn_block = &s.unwrap().block;

    let item_struct = &arg_struct.unwrap().path().get_ident().unwrap().clone();
    let item_cap = &arg_capability.unwrap().path().get_ident().unwrap().clone();
    let capability = format_ident!("{}{}{}", CAP_PREFIX, item_cap, item_struct);

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
            // this need to be dynamic
            // we need to find the CapRead"Struct_name"
            // action: #item<#item_struct> in some cases should be #item_cap<#id_type>
            // and get its values to be put in here.
            async fn perform(&self, action: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                #fn_block
            }
        }
    };
    out.into()
}
