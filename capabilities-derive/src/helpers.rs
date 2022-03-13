use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, Item, ItemStruct, Lit, Meta, MetaNameValue, NestedMeta, Type};

#[allow(dead_code)]
const FIELD_NAME: &str = "con";

fn get_id_identifier() -> Ident {
    format_ident!("{}", "id")
}

fn get_name_identifier() -> Ident {
    format_ident!("{}", "name")
}

fn get_cap_macro() -> TokenStream2 {
    quote! {
        macro_rules! cap {
        ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
            #[async_trait]
            pub trait $name: $(CapabilityTrait<$operation, Data = $d, Error = $e>+)+ {}

            #[async_trait]
            impl $name for $type {}
        };
    }}
}

pub fn parse_service_field_for_name(attr_args: &Vec<NestedMeta>) -> Option<MetaNameValue> {
    let mut id_vec = vec![];
    for i in attr_args {
        let m = match i {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(nv) => {
                    let name = get_name_identifier();
                    if nv.path.get_ident().unwrap().eq(&name) {
                        Some(nv)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
        if m.is_some() {
            let val = m.unwrap();
            id_vec.push(val);
        }
    }
    if id_vec.is_empty() {
        None
    } else {
        let val = id_vec.pop().unwrap().to_owned();
        Some(val)
    }
}

pub fn parse_field_args_for_id(attr_args: &Vec<NestedMeta>) -> Option<MetaNameValue> {
    let mut id_vec = vec![];
    for i in attr_args {
        let m = match i {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(nv) => {
                    let id = get_id_identifier();
                    if nv.path.get_ident().unwrap().eq(&id) {
                        Some(nv)
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        };
        if m.is_some() {
            let val = m.unwrap();
            id_vec.push(val);
        }
    }
    if id_vec.is_empty() {
        None
    } else {
        let val = id_vec.pop().unwrap().to_owned();
        Some(val)
    }
}

pub fn parse_metavalue_for_type(
    id_metavalue: &Option<MetaNameValue>,
    item_struct: &ItemStruct,
) -> Option<Type> {
    let out = if id_metavalue.is_none() {
        None
    } else {
        let mut id_type = vec![];
        let id_field_name = match &id_metavalue.as_ref().unwrap().lit {
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
    };
    out
}

#[allow(dead_code)]
pub fn parse_metavalue_for_type_ident(
    id_metavalue: &Option<MetaNameValue>,
    item_struct: &Ident,
) -> Option<Ident> {
    let out = if id_metavalue.is_none() {
        Some(item_struct.to_owned())
    } else {
        let id_field_name = match &id_metavalue.as_ref().unwrap().lit {
            Lit::Str(a) => Some(a.value()),
            _ => None,
        };
        if id_field_name.is_some() {
            let ident_fieldname = format_ident!("{}", &id_field_name.unwrap());
            Some(ident_fieldname)
        } else {
            None
        }
    };
    out
}

pub fn get_id_type(id_metavalue: &Option<MetaNameValue>, item_struct: &Ident) -> Option<Ident> {
    let out = if id_metavalue.is_none() {
        Some(item_struct.to_owned())
    } else {
        Some(format_ident!("{}Id", item_struct))
    };
    out
}

fn get_ident_from_field_name(field_name: Option<MetaNameValue>) -> Ident {
    let id_field_name = if field_name.is_some() {
        match &field_name.as_ref().unwrap().lit {
            Lit::Str(a) => Some(a.value()),
            _ => None,
        }
    } else {
        None
    };
    let field_name = format_ident!("{}", id_field_name.unwrap_or(FIELD_NAME.to_string()));
    field_name
}

pub fn impl_code_database(
    service_token: &Meta,
    item: Item,
    field_name: Option<MetaNameValue>,
) -> TokenStream {
    let field_id = get_ident_from_field_name(field_name);

    let out = quote! {
        use sqlx::Pool;
        use async_trait::async_trait;
        use ::capabilities::Capability;

        #[derive(Clone)]
        pub struct CapService {
            #field_id: #service_token,
        }

        #[derive(Debug)]
        pub struct CapServiceError;

        impl CapService {
            pub async fn build(conf: String) -> Result<Self, crate::CapServiceError> {
                let con = Pool::connect(&conf)
                    .await
                    .expect("Failed to connect database");

                Ok ( Self { #field_id: con })
            }
        }
        #[async_trait]
        pub trait CapabilityTrait<Operation> {
            type Data;
            type Error;
            async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
        }
        pub trait CapToEnum {
            fn into_enum(&self) -> Capability;
        }
        #item
    };
    out.into()
}

pub fn impl_code_webservice(
    service_token: &Meta,
    item: Item,
    field_name: Option<MetaNameValue>,
) -> TokenStream {
    let field_id = get_ident_from_field_name(field_name);

    let out = quote! {
        use async_trait::async_trait;
        use ::capabilities::Capability;

        #[derive(Clone)]
        pub struct CapService {
            #field_id: #service_token,
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
        pub trait CapabilityTrait<Operation> {
            type Data;
            type Error;
            async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
        }
        pub trait CapToEnum {
            fn into_enum(&self) -> Capability;
        }
        #item
    };

    out.into()
}

pub fn generate_caps(
    capabilities: &Vec<Ident>,
    id_type: Option<Type>,
    struct_name: &Ident,
) -> TokenStream2 {
    let create = format_ident!("{}{}", "CapCreate", struct_name).to_string();
    let read = format_ident!("{}{}", "CapRead", struct_name).to_string();
    let update = format_ident!("{}{}", "CapUpdate", struct_name).to_string();
    let delete = format_ident!("{}{}", "CapDelete", struct_name).to_string();
    let readall = format_ident!("{}{}", "CapReadAll", struct_name).to_string();
    let deleteall = format_ident!("{}{}", "CapDeleteAll", struct_name).to_string();
    let updateall = format_ident!("{}{}", "CapUpdateAll", struct_name).to_string();
    let idstruct = format_ident!("{}Id", struct_name);

    let mut tokens = vec![];
    let capmacro = get_cap_macro();
    for cap in capabilities {
        let capid = format_ident!("{}{}", cap.to_string(), "Id");
        let outtokens = if cap.to_string().eq(&create) {
            Some(quote! {
                #capmacro
                cap!( #cap for CapService, composing { Create<#struct_name>, #struct_name, CapServiceError});
                
                impl CapToEnum for Create<#struct_name> {
                    fn into_enum(&self) -> Capability {
                        Capability::Create
                    }
                }
            })
        } else if cap.to_string().eq(&read) {
            if id_type.is_some() {
                Some(quote! {
                    #capmacro

                    cap!( #capid for CapService, composing { Read<#idstruct>, #struct_name, CapServiceError});
                    cap!( #cap for CapService, composing { Read<#struct_name>, #struct_name, CapServiceError});
                    impl CapToEnum for Read<#struct_name> {
                        fn into_enum(&self) -> ::capabilities::Capability {
                            ::capabilities::Capability::Read
                        }
                    }

                    impl CapToEnum for Read<#idstruct> {
                        fn into_enum(&self) -> ::capabilities::Capability {
                            ::capabilities::Capability::Read
                        }
                    }
                })
            } else if id_type.is_none() {
                Some(quote! {
                   #capmacro
                    cap!( #cap for CapService, composing { Read<#struct_name>, #struct_name, CapServiceError});
                    impl CapToEnum for Read<#struct_name> {
                        fn into_enum(&self) -> ::capabilities::Capability {
                            ::capabilities::Capability::Read
                        }
                    }
                })
            } else {
                None
            }
        } else if cap.to_string().eq(&update) {
            if id_type.is_some() {
                Some(quote! {
                    #capmacro

                    cap!( #capid for CapService, composing { Update<#idstruct>, (), CapServiceError});
                    cap!( #cap for CapService, composing { Update<#struct_name>, (), CapServiceError});
                    impl CapToEnum for Update<#struct_name> {
                        fn into_enum(&self) -> Capability {
                            Capability::Update
                        }
                    }
                    impl CapToEnum for Update<#idstruct> {
                        fn into_enum(&self) -> Capability {
                            Capability::Update
                        }
                    }
                    
                })
            } else if id_type.is_none() {
                Some(quote! {
                    #capmacro
                    cap!( #cap for CapService, composing { Update<#struct_name>, (), CapServiceError});
                    impl CapToEnum for Update<#struct_name> {
                        fn into_enum(&self) -> ::capabilities::Capability {
                            ::capabilities::Capability::Update
                        }
                    }
                })
            } else {
                None
            }
        } else if cap.to_string().eq(&delete) {
            if id_type.is_some() {
                Some(quote! {
                    #capmacro

                    cap!( #capid for CapService, composing { Delete<#idstruct>, (), CapServiceError});
                    cap!( #cap for CapService, composing { Delete<#struct_name>, (), CapServiceError});
                    impl CapToEnum for Delete<#struct_name> {
                        fn into_enum(&self) -> Capability {
                            Capability::Delete
                        }
                    }

                    impl CapToEnum for Delete<#idstruct> {
                        fn into_enum(&self) -> Capability {
                            Capability::Delete
                        }
                    }
                })
            } else if id_type.is_none() {
                Some(quote! {
                   #capmacro
                    cap!( #cap for CapService, composing { Delete<#struct_name>, (), CapServiceError});
                    impl CapToEnum for Delete<#struct_name> {
                        fn into_enum(&self) -> ::capabilities::Capability {
                            ::capabilities::Capability::Delete
                        }
                    }
                })
            } else {
                None
            }
        } else if cap.to_string().eq(&deleteall) {
            Some(quote! {
                #capmacro
                use capabilities::EmptyInput;
                cap!( #cap for CapService, composing { DeleteAll<Vec<#struct_name>>, (), CapServiceError});
                impl CapToEnum for DeleteAll<Vec<#struct_name>> {
                    fn into_enum(&self) -> Capability {
                        Capability::DeleteAll
                    }
                }
            })
        } else if cap.to_string().eq(&updateall) {
            Some(quote! {
                #capmacro
                use capabilities::EmptyInput;
                cap!( #cap for CapService, composing { UpdateAll<Vec<#struct_name>>, (), CapServiceError});
                impl CapToEnum for UpdateAll<Vec<#struct_name>> {
                    fn into_enum(&self) -> Capability {
                        Capability::UpdateAll
                    }
                }
            })
        } else if cap.to_string().eq(&readall) {
            // Lets try EmptyInput
            Some(quote! {
                #capmacro
                use capabilities::EmptyInput;
                cap!( #cap for CapService, composing { ReadAll<#struct_name>, Vec<#struct_name>, CapServiceError});
                impl CapToEnum for ReadAll<Vec<#struct_name>> {
                    fn into_enum(&self) -> Capability {
                        Capability::ReadAll
                    }
                }
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
