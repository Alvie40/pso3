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
        // Páginas estáticas/templates
        .at("/", get(pages::login_page))
        .at("/login", get(pages::login_page).post(login))  // This will handle both GET and POST
        .at("/cadastro", get(pages::cadastro_page))
        .at("/chat", get(pages::chat_page).with(ChatMiddleware))
        
        // Rotas de autenticação
        .at("/auth/register", post(register))
        .at("/auth/me", get(me))
        .at("/auth/logout", get(logout))
        
        // Área administrativa
        .nest("/admin", 
            Route::new()
                .at("/", get(pages::admin_page))
                .with(AdminMiddleware)
        )
}
