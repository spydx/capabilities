
#[allow(unused_macros)]
macro_rules! database_cap {
    ($name:ident) => {
        impl #ident {
            pub async build() -> Result<Self, std::io::Error> {}
        }
    }
}

#[async_trait]
pub trait Capability<Operation> {
    type Data;
    type Error;
    async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
}


#[allow(unused_macros)]
macro_rules! cap {
    ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
        #[async_trait]
        pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}

        #[async_trait]
        impl $name for $type {}
    };
}