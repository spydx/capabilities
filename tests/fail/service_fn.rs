use capabilities_derive::service;

struct User {
    pub name: String,
}

struct UserError;

#[service]
fn create_user() -> Result<User, UserError> {
    Ok(User { name: "Kenneth".to_string() })
}

fn main() {

}