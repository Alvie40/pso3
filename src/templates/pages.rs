use poem::web::Html;
use std::fs;
use poem::handler;

#[handler]
pub fn login_page() -> Html<&'static str> {
    Html(Box::leak(
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/login.html"))
            .unwrap_or_else(|_| "<h1>Error loading login template</h1>".to_string())
            .into_boxed_str(),
    ))
}

#[handler]
pub fn cadastro_page() -> Html<&'static str> {
    Html(Box::leak(
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/cadastro.html"))
            .unwrap_or_else(|_| "<h1>Error loading cadastro template</h1>".to_string())
            .into_boxed_str(),
    ))
}

#[handler]
pub fn chat_page() -> Html<&'static str> {
    Html(Box::leak(
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/chat.html"))
            .unwrap_or_else(|_| "<h1>Error loading chat template</h1>".to_string())
            .into_boxed_str(),
    ))
}

#[handler]
pub fn admin_page() -> Html<&'static str> {
    Html(Box::leak(
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/admin.html"))
            .unwrap_or_else(|_| "<h1>Error loading admin template</h1>".to_string())
            .into_boxed_str(),
    ))
}

pub fn not_found_page() -> Html<&'static str> {
    Html(Box::leak(
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/templates/404.html"))
            .unwrap_or_else(|_| "<h1>Error loading 404 template</h1>".to_string())
            .into_boxed_str(),
    ))
}
