use capabilities_derive::capability;

#[capability]
struct Database {
    pub _db: String,
}

#[allow(dead_code)]
fn main() {

}