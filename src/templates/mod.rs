use poem::{endpoint::StaticFilesEndpoint, Route};

pub mod pages;

pub use pages::*;

pub fn templates_route() -> Route {
    Route::new().nest("/", StaticFilesEndpoint::new("templates").show_files_listing())
}
