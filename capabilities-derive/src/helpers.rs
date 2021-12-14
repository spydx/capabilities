use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, Item, ItemStruct, Lit, Meta, MetaNameValue, NestedMeta, Type};

fn get_id_identifier() -> Ident {
    format_ident!("{}", "id")
}

fn get_name_identifier() -> Ident {
    format_ident!("{}", "name")
}

pub fn get_cap_macro() -> TokenStream2 {
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

pub fn impl_code_database(service_token: Meta, item: Item) -> TokenStream {
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

pub fn impl_code_webservice(service_token: Meta, item: Item) -> TokenStream {
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
