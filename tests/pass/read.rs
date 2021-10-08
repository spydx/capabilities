use capabilities_proc_macro::capability;

#[capability(Create, Read, Update)]
struct User {
    pub name: String,
    pub password: String,
}


#[capability(All)]
struct Users {
    pub name: String,
    pub password: String,
}

fn main() {

}