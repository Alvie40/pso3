use poem::{
    middleware::AddData,
    web::Html,
    EndpointExt,
    IntoEndpoint,
    Route,
    get,
    post,
};
use crate::{
    auth::{
        handler::{login, register, me, logout},
        AdminMiddleware,
        ChatMiddleware,
    },
    state::AppState,
    templates::pages,
    api::Api,
};

pub fn routes(state: AppState) -> Route {
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
        
        // API routes
        .nest("/api", 
            Route::new()
                .nest("/", Api::new(state).into_endpoint())
        )
        
        // Admin routes
        .nest("/admin", 
            Route::new()
                .at("/", get(pages::admin_page))
                .with(AdminMiddleware)
        )
}
