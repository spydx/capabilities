#![feature(proc_macro_diagnostic)]
mod helpers;

use helpers::{
    generate_caps, impl_code_database, impl_code_webservice, parse_field_args_for_id,
    parse_metavalue_for_type, parse_metavalue_for_type_ident, parse_service_field_for_name,
};
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::FnArg::Typed;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta};
use syn::{Block, Ident};

const POOL_SQLITE: &str = "SqliteDb";
const POOL_POSTGRES: &str = "PostgresDb";
const WEB_SERVICE: &str = "WebService";
const CAP_PREFIX: &str = "Cap";

/*
   Need better error handling for when user types wrong paramteres.
   E.g writing #[service(name = "db")], results in unwrap() on a None.
   We are missing the Service type param, and should give this message to the user.

   Database needs sqlx::Pool injected in the code.. fixed now but not sure this is the best way.

*/
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
                        .error(format!("Incorrect order, missing ServiceType: {}", ident))
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
    let service_token = if service_type.is_some() {
        Some(service_type.unwrap().unwrap())
    } else {
        service_type
            .span()
            .unstable()
            .error(format!("Missing Service type"))
            .emit();
        None
    };
    let service_field = parse_service_field_for_name(&input_args);

    let out = match service_token
        .as_ref()
        .unwrap()
        .path()
        .get_ident()
        .unwrap()
        .to_string()
        .as_str()
    {
        POOL_SQLITE => Some(impl_code_database(
            &service_token.unwrap(),
            item,
            service_field,
        )),
        POOL_POSTGRES => Some(impl_code_database(
            &service_token.unwrap(),
            item,
            service_field,
        )),
        WEB_SERVICE => Some(impl_code_webservice(
            &service_token.unwrap(),
            item,
            service_field,
        )),
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
    let id_type = parse_metavalue_for_type(&id_metavalue.clone(), &item_struct);

    let generated_caps = generate_caps(&capidents, id_type.clone(), &struct_id);
    
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
        s.span().unstable().error("Please implement your function").emit();
    }

    let arg_path = if attr_args.len() == 3 {
        attr_args.pop()
    } else {
        None
    };

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
                }
                _ => None,
            },
            _ => None,
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
        arg_capability
            .span()
            .unstable()
            .error("Missing capability / struct / wrong order")
            .emit();
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
        arg_capability
            .span()
            .unstable()
            .error("Missing capability / struct / wrong order")
            .emit();
        None
    };

    /*let fn_signature =  if s.is_some() {
        Some(&s.unwrap().sig.ident)
    } else {
        None
    };*/
    let fn_signature = &s.unwrap().sig.ident;

    let fn_attr = if s.unwrap().sig.inputs.first().is_some() {
        s.unwrap().sig.inputs.first()
    } else {
        None
    };

    let fn_attrname = if fn_attr.is_some() { 
            match fn_attr.unwrap() {
                Typed(t) => {              
                    let ident = &t.pat;
                    Some(ident)
                }
                _ => None,
        }
    } else {
        None
    };

    let fn_block = &s.unwrap().block;

    let item_struct = if arg_struct.is_some() {
        arg_struct.unwrap().path().get_ident().unwrap().clone()
    } else {
        format_ident!("{}", "ErrorIdentStruct")
    };

    let item_cap = if arg_capability.is_some() {
        arg_capability.unwrap().path().get_ident().unwrap().clone()
    } else {
        format_ident!("{}", "CapErrorIdent")
    };
    
    let capability: Ident = if arg_path.is_none() {
        format_ident!("{}{}{}", CAP_PREFIX, item_cap, item_struct)
    } else {
        format_ident!("{}{}{}{}", CAP_PREFIX,  item_cap, item_struct, "Id")
    };

    // this needs to switch if it is a ReadAll.. Should be () then.. or a new EmptyInput type?
    let action_id = parse_metavalue_for_type_ident(&arg_path, &item_struct);
    
    let out = if capability.to_string().contains("ReadAll") {
        //let action_struct = proc_macro2::Ident::new("EmptyInput", Span::call_site());
        let action_struct = format_ident!("EmptyInput");
        let out = impl_readall_function_trait(
            fn_signature,
            action_struct,
            item_struct,
            item_cap,
            capability,
            fn_block,
        );
        out.into()
    } else if capability.to_string().contains("UpdateAll") {
        
        let out = impl_updateall_function_trait(
            fn_signature,
            item_struct,
            item_cap,
            fn_attrname,
            capability,
            fn_block,
        );
        out.into()
    }
    else {
        let action_struct = action_id.as_ref().unwrap().to_owned();
        let out = quote! {
    
            pub async fn #fn_signature<Service>(service: &Service, param: #action_struct) -> Result<#item_struct, CapServiceError>
            where
                Service: #capability,
            {
                service.perform(::capabilities::#item_cap { data: param }).await
            }

            #[async_trait]
            impl Capability<#item_cap<#action_struct>> for CapService {
                type Data = #item_struct;
                type Error = CapServiceError;

                async fn perform(&self, action: #item_cap<#action_id>) -> Result<Self::Data, Self::Error> {
                    let #fn_attrname = action.data;
                    #fn_block
                }
            }
        };
        out.into()
    };

    out
}

fn impl_readall_function_trait(
    fn_signature: &Ident,
    _action_struct: Ident,
    item_struct: Ident,
    item_cap: Ident,
    capability: Ident,
    fn_block: &Box<Block>,
) -> TokenStream {
    let out = quote! {
        
        pub async fn #fn_signature<Service>(service: &Service, param: #item_struct) -> Result<Vec<#item_struct>, CapServiceError>
        where
            Service: #capability,
        {
            service.perform(::capabilities::#item_cap { data: param }).await
        }

        #[async_trait]
        impl Capability<#item_cap<#item_struct>> for CapService {
            type Data = Vec<#item_struct>;
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                #fn_block
            }
        }
    };
    out.into()
}


use syn::Pat;

fn impl_updateall_function_trait(
    fn_signature: &Ident,
    item_struct: Ident,
    item_cap: Ident,
    fn_attrname: Option<&Box<Pat>>,
    capability: Ident,
    fn_block: &Box<Block>,
) -> TokenStream {
    let out = quote! {
        
        pub async fn #fn_signature<Service>(service: &Service, param: Vec<#item_struct>) -> Result<Vec<#item_struct>, CapServiceError>
        where
            Service: #capability,
        {
            service.perform(::capabilities::#item_cap { data: param }).await
        }

        #[async_trait]
        impl Capability<#item_cap<Vec<#item_struct>>> for CapService {
            type Data = Vec<#item_struct>;
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                let #fn_attrname = action.data;
                #fn_block
            }
        }
    };
    out.into()
}