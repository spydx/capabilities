use capabilities_proc_macro::capability;


// Create, Read, Update, Delete, ReadAll, UpdateAll, DeleteAll, All
#[capability(Create)]
struct User {
    pub name: String,
    pub password: String,
}


#[capability(All)]
struct Accounts {
    pub name: String,
    pub dallaz: String,
}

fn main() {

}