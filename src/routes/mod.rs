use poem::{get, post, Route};

use crate::auth::handler::{login, me};

pub fn create_routes() -> Route {
    Route::new()
        .at("/login", post(login))
        .at("/me", get(me))
}
