use async_trait::async_trait;

#[async_trait]
pub trait Capability<Operation> {
    type Data;
    type Error;

    async fn perform(&self, _: Operation) -> Result<Self::Data, Self::Error>;
}

macro_rules! cap {
    ($name:ident for $type:ty, composing $({$operation:ty, $d:ty, $e:ty}),+) => {
        #[async_trait]
        pub trait $name: $(Capability<$operation, Data = $d, Error = $e>+)+ {}

        #[async_trait]
        impl $name for $type {}
    };
}
struct Orders {
    id: i32, 
    name: String,
}

struct Database;
struct DatabaseError;

struct Read<T>{data: T}

cap!(CanReadOrders for Database, composing { Read<String>, Orders, DatabaseError} );


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    
    Ok(())
}
