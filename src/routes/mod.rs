use poem::{
    get, post, Route, EndpointExt,
};
use crate::{
    templates::pages, 
    auth::{
        handler::{login, register, me, api_login, logout},
        AdminMiddleware, ChatMiddleware
    },
};

pub fn routes() -> Route {
    Route::new()
        // Public routes
        .at("/", get(pages::login_page))
        .at("/login", get(pages::login_page).post(login))
        .at("/cadastro", get(pages::cadastro_page))
        .at("/auth/register", post(register))
        
        // Protected chat route
        .at("/chat", get(pages::chat_page).with(ChatMiddleware))
        
        // Auth routes
        .at("/auth/me", get(me))
        .at("/auth/logout", get(logout))
        
        // Admin routes
        .nest("/admin", 
            Route::new()
                .at("/", get(pages::admin_page))
                .with(AdminMiddleware)
        )
}
