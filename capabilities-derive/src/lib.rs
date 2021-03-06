#![feature(proc_macro_diagnostic)]
mod helpers;

use helpers::{
    generate_caps, get_id_type, impl_code_database, impl_code_webservice, parse_field_args_for_id,
    parse_metavalue_for_type, parse_service_field_for_name,
};
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::FnArg::Typed;
use syn::{parse_macro_input, AttributeArgs, Item, Meta, NestedMeta};
use syn::{Block, Ident, Pat};

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
    let service_token = if let Some(service_type) = service_type {
        Some(service_type.unwrap())
    } else {
        service_type
            .span()
            .unstable()
            .error("Missing Service type")
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
            NestedMeta::Meta(Meta::Path(p)) => Some(p),
            _ => None,
        };
        if let Some(val) = m {
            caps.push(val)
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
    let id_type = parse_metavalue_for_type(&id_metavalue, &item_struct);
    let typealias = format_ident!("{}Id", struct_id);
    let generated_caps = generate_caps(&capidents, id_type.clone(), struct_id);

    // #( use ::capabilities::#caps;)*
    quote! {
        #item_struct
        pub struct #typealias{ id: #id_type }
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
        s.span()
            .unstable()
            .error("Please implement your function")
            .emit();
    }

    let arg_path = if attr_args.len() == 3 {
        attr_args.pop()
    } else {
        None
    };

    let arg_struct = attr_args.pop();
    let arg_capability = attr_args.pop();

    let arg_path = if let Some(arg_path) = arg_path {
        match arg_path {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                    let field_name = format_ident!("{}", "id");
                    if nv.path.get_ident().unwrap().eq(&field_name) {
                        Some(nv)
                    } else {
                        None
                    }
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

    let arg_struct = if let Some(arg_struct) = arg_struct {
        match arg_struct {
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

    let fn_signature = &s.unwrap().sig.ident;

    let fn_attr = if s.unwrap().sig.inputs.first().is_some() {
        s.unwrap().sig.inputs.first()
    } else {
        None
    };

    let fn_attrname = if let Some(fn_attr) = fn_attr {
        match fn_attr {
            Typed(t) => {
                
                let ident = t.pat.as_ref();
                Some(ident)
            }
            _ => None,
        }
    } else {
        None
    };

    let _fn_attrtype = if let Some(fn_attr) = fn_attr {
        match fn_attr {
            Typed(t) => {
                let attrtype = t.ty.as_ref();
                Some(attrtype)
            }
            _ => None,
        }
    } else {
        None
    };

    let fn_block = &s.unwrap().block;

    let item_struct = if let Some(arg) = arg_struct {
        arg.path().get_ident().unwrap().clone()
    } else {
        format_ident!("{}", "ErrorIdentStruct")
    };

    let item_cap = if let Some(cap) = arg_capability {
        cap.path().get_ident().unwrap().clone()
    } else {
        format_ident!("{}", "CapErrorIdent")
    };

    let capability: Ident = if arg_path.is_none() {
        format_ident!("{}{}{}", CAP_PREFIX, item_cap, item_struct)
    } else {
        format_ident!("{}{}{}{}", CAP_PREFIX, item_cap, item_struct, "Id")
    };

    // this needs to switch if it is a ReadAll.. Should be () then.. or a new EmptyInput type?
    let action_id = get_id_type(&arg_path, &item_struct);

    let out = if capability.to_string().contains("ReadAll") {
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
    } else if capability.to_string().contains("DeleteAll") {
        if fn_attrname.is_none() {
            fn_attrname
                .span()
                .unstable()
                .error("Missing argument for function, pass in the data you are deleting")
                .emit();
        }

        let out = impl_deleteall_function_trait(
            fn_signature,
            item_struct,
            item_cap,
            fn_attrname,
            capability,
            fn_block,
        );
        out.into()
    } else if 
        capability.to_string().eq(&format!("{}{}{}", CAP_PREFIX, "Delete", item_struct)) 
    {
        let out = impl_delete_function_trait(
            fn_signature,
            item_struct,
            item_cap,
            fn_attrname,
            capability,
            fn_block,
        );

        out.into()
    } else if 
    capability.to_string().eq(&format!(
        "{}{}{}{}",
        CAP_PREFIX, "Delete", item_struct, "Id"
    )) {
        let out = impl_delete_function_trait(
            fn_signature,
            action_id.unwrap(),
            item_cap,
            fn_attrname,
            capability,
            fn_block
        );
        out.into()
    }
     else if capability
        .to_string()
        .eq(&format!("{}{}{}", CAP_PREFIX, "Update", item_struct))
    {
        let out = impl_update_function_trait(
            fn_signature,
            item_struct,
            item_cap,
            fn_attrname,
            capability,
            fn_block,
        );
        out.into()
    } else if  capability.to_string().eq(&format!(
        "{}{}{}{}",
        CAP_PREFIX, "Update", item_struct, "Id"
    )) {
        let out = impl_update_function_trait(
            fn_signature,
            action_id.unwrap(),
            item_cap,
            fn_attrname,
            capability,
            fn_block);

        out.into()
    } else {
        let action_struct = action_id.as_ref().unwrap().to_owned();
        let out = quote! {

            pub async fn #fn_signature<Service>(service: &Service, param: #action_struct, cap: ::capabilities::Capability) -> Result<#item_struct, CapServiceError>
            where
                Service: #capability,
            {
                let valid = ::capabilities::#item_cap { data: param };
                if valid.into_enum().eq(&cap) {
                     service.perform(valid).await
                } else {
                    Err(CapServiceError)
                }
            }

            #[async_trait]
            impl CapabilityTrait<#item_cap<#action_struct>> for CapService {
                type Data = #item_struct;
                type Error = CapServiceError;

                async fn perform(&self, action: #item_cap<#action_id>) -> Result<Self::Data, Self::Error> {
                    let #fn_attr = action.data;
                    #fn_block
                }
            }
        };
        out
    };

    out.into()
}

fn impl_readall_function_trait(
    fn_signature: &Ident,
    _action_struct: Ident,
    item_struct: Ident,
    item_cap: Ident,
    capability: Ident,
    fn_block: &Block,
) -> TokenStream {
    let out = quote! {

        pub async fn #fn_signature<Service>(service: &Service, cap: ::capabilities::Capability) -> Result<Vec<#item_struct>, CapServiceError>
        where
            Service: #capability,
        {
            let param: Vec<#item_struct> = Vec::<#item_struct>::new();
            let valid = ::capabilities::#item_cap { data: param };
            if valid.into_enum().eq(&cap) {
                service.perform(valid).await
            } else {
                Err(CapServiceError)
            }

        }

        #[async_trait]
        impl CapabilityTrait<#item_cap<Vec<#item_struct>>> for CapService {
            type Data = Vec<#item_struct>;
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<Vec<#item_struct>>) -> Result<Self::Data, Self::Error> {
                #fn_block
            }
        }
    };
    out.into()
}

fn impl_updateall_function_trait(
    fn_signature: &Ident,
    item_struct: Ident,
    item_cap: Ident,
    fn_attrname: Option<&Pat>,
    capability: Ident,
    fn_block: &Block,
) -> TokenStream {
    let out = quote! {

        pub async fn #fn_signature<Service>(service: &Service, param: Vec<#item_struct>, cap: ::capabilities::Capability) -> Result<(), CapServiceError>
        where
            Service: #capability,
        {
            let valid = ::capabilities::#item_cap { data: param };
            if valid.into_enum().eq(&cap) {
                service.perform(valid).await
            } else {
                Err(CapServiceError)
            }
        }

        #[async_trait]
        impl CapabilityTrait<#item_cap<Vec<#item_struct>>> for CapService {
            type Data = ();
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<Vec<#item_struct>>) -> Result<Self::Data, Self::Error> {
                let #fn_attrname = action.data;
                #fn_block
            }
        }
    };
    out.into()
}

fn impl_deleteall_function_trait(
    fn_signature: &Ident,
    item_struct: Ident,
    item_cap: Ident,
    fn_attrname: Option<&Pat>,
    capability: Ident,
    fn_block: &Block,
) -> TokenStream {
    let data_accessor = if fn_attrname.is_some() {
        quote! { let #fn_attrname = action.data; }
    } else {
        quote! {}
    };

    let out = quote! {

        pub async fn #fn_signature<Service>(service: &Service, param: Vec<#item_struct>, cap: ::capabilities::Capability) -> Result<(), CapServiceError>
        where
            Service: #capability,
        {
            let valid = ::capabilities::#item_cap { data: param };
            if valid.into_enum().eq(&cap) {
                service.perform(valid).await
            } else {
                Err(CapServiceError)
            }
        }

        #[async_trait]
        impl CapabilityTrait<#item_cap<(Vec<#item_struct>)>> for CapService {
            type Data = ();
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<Vec<#item_struct>>) -> Result<Self::Data, Self::Error> {
                #data_accessor
                #fn_block
            }
        }
    };
    out.into()
}

fn impl_delete_function_trait(
    fn_signature: &Ident,
    item_struct: Ident,
    item_cap: Ident,
    fn_attrname: Option<&Pat>,
    capability: Ident,
    fn_block: &Block,
) -> TokenStream {
    let _typealias = format_ident!("{}Id", item_struct);
    //println!("{:#?}: {:#?}",action_struct,  _typealias);

    let data_accessor = if fn_attrname.is_some() {
        quote! { let #fn_attrname = action.data; }
    } else {
        quote! {}
    };
    let out = quote! {

        pub async fn #fn_signature<Service>(service: &Service, param: #item_struct, cap: ::capabilities::Capability) -> Result<(), CapServiceError>
        where
            Service: #capability,
        {
            let valid = ::capabilities::#item_cap { data: param };
            if valid.into_enum().eq(&cap) {
                service.perform(valid).await
            } else {
                Err(CapServiceError)
            }
        }

        #[async_trait]
        impl CapabilityTrait<#item_cap<#item_struct>> for CapService {
            type Data = ();
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                #data_accessor
                #fn_block
            }
        }
    };
    out.into()
}

fn impl_update_function_trait(
    fn_signature: &Ident,
    item_struct: Ident,
    item_cap: Ident,
    fn_attrname: Option<&Pat>,
    capability: Ident,
    fn_block: &Block,
) -> TokenStream {
    let out = quote! {

        pub async fn #fn_signature<Service>(service: &Service, param: #item_struct, cap: ::capabilities::Capability) -> Result<(), CapServiceError>
        where
            Service: #capability,
        {
            let valid = ::capabilities::#item_cap { data: param };
            if valid.into_enum().eq(&cap) {
                service.perform(valid).await
            } else {
                Err(CapServiceError)
            }
        }

        #[async_trait]
        impl CapabilityTrait<#item_cap<#item_struct>> for CapService {
            type Data = ();
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                let #fn_attrname = action.data;
                #fn_block
            }
        }
    };
    out.into()
}

fn _impl_deleteid_function_trait(
    fn_signature: &Ident,
    item_struct: Ident,
    item_cap: Ident,
    fn_attrname: Option<&Pat>,
    capability: Ident,
    fn_block: &Block,
) -> TokenStream {
    let data_accessor = if fn_attrname.is_some() {
        quote! { let #fn_attrname = action.data; }
    } else {
        quote! {}
    };
    let out = quote! {

        pub async fn #fn_signature<Service>(service: &Service, param: #item_struct, cap: Capability) -> Result<(), CapServiceError>
        where
            Service: #capability,
        {
            let valid = ::capabilities::#item_cap { data: param };
            if valid.into_enum().eq(&cap) {
                service.perform(valid).await
            } else {
                Err(CapServiceError)
            }
        }

        #[async_trait]
        impl CapabilityTrait<#item_cap<#item_struct>> for CapService {
            type Data = ();
            type Error = CapServiceError;

            async fn perform(&self, action: #item_cap<#item_struct>) -> Result<Self::Data, Self::Error> {
                #data_accessor
                #fn_block
            }
        }
    };
    out.into()
}
