use rocket::http::Status;
use rocket::response::status;

pub fn is_allowed_characters(host: String) -> bool {
    host.chars()
        .all(|c| ('0' <= c && c <= '9') || ('a' <= c && c <= 'f') || c == '-')
}

pub fn get_subdomain(host: String) -> Result<String, status::Custom<String>> {
    if cfg!(debug_assertions) {
        dbg!(&host);
    }

    let list: Vec<&str> = host.split('.').collect();
    match list.get(0) {
        Some(s) => Ok(s.to_string()),
        None => Err(status::Custom(
            Status::NotFound,
            "couldn't find subdomain".to_string(),
        )),
    }
}
