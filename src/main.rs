use std::env;

mod alertmanager;

fn main() {
    println!("Hello, world!");

    let ntfy_user = match env::var("NTFY_USER") {
        Ok(user) => user,
        Err(err) => panic!("{}", err),
    };

    let ntfy_password = match env::var("NTFY_PASSWORD") {
        Ok(user) => user,
        Err(err) => panic!("{}", err),
    };
}
