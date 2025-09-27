pub mod bounding_box;

pub fn url_to_ref(url: &str) -> Option<String> {
    url.split('/')
        .next_back()
        .and_then(|last_part| last_part.split('.').next())
        .map(|s| s.to_string())
}
