#[cfg(feature = "ssr")]
use leptos::prelude::{
    AutoReload, ElementChild, GlobalAttributes, HydrationScripts, IntoMaybeErased, IntoView, LeptosOptions, view,
};

pub mod components;
pub mod icons;

#[cfg(feature = "ssr")]
pub mod ssr;

mod server_functions;

pub mod meta {
    pub use leptos_meta::{Title, provide_meta_context};
}

pub mod prelude {
    pub use leptos::prelude::{
        AddAnyAttr, Children, ClassAttribute, CustomAttribute, ElementChild, Get, IntoAnyAttribute, IntoMaybeErased,
        IntoView, ServerFnError, Signal, component, server, view,
    };

    pub use leptos_fluent::{move_tr, tr};
}

pub mod router {
    pub use leptos_router::StaticSegment;
    pub use leptos_router::components::{Route, Router, Routes};
}

#[cfg(feature = "hydrate")]
pub fn hydrate_body<IV>(app_fn: fn() -> IV)
where
    IV: leptos::prelude::IntoView + 'static,
{
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(app_fn)
}

#[cfg(feature = "ssr")]
fn shell_with_app<IV>(options: LeptosOptions, app_fn: fn() -> IV) -> impl IntoView
where
    IV: IntoView + 'static,
{
    use leptos_meta::{HashedStylesheet, MetaTags};

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <MetaTags />
                <HashedStylesheet id="leptos" options=options />
            </head>
            <body>{app_fn()}</body>
        </html>
    }
}

#[cfg(feature = "ssr")]
pub async fn serve<IV>(leptos_options: LeptosOptions, app_fn: fn() -> IV)
where
    IV: IntoView + 'static,
{
    use axum::Router;
    use cookie::{Key, SameSite};
    use fred::prelude::{ClientLike, Config, Pool};
    use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
    use time::Duration;
    use tokio::net::TcpListener;
    use tower_sessions::{Expiry, SessionManagerLayer};
    use tower_sessions_redis_store::RedisStore;

    use crate::ssr::config::{SESSION_CONFIG, load_config};

    load_config();

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(app_fn);
    let redis_pool = Pool::new(
        Config::from_url(&SESSION_CONFIG.redis_url).expect("Could not get Redis URL for session."),
        None,
        None,
        None,
        10,
    )
    .expect("Could not get Redis pool for session.");

    let redis_conn = redis_pool.connect();
    redis_pool
        .wait_for_connect()
        .await
        .expect("Could not get Redis connection for session.");

    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_domain(SESSION_CONFIG.domain.clone())
        .with_expiry(Expiry::OnInactivity(Duration::days(30)))
        .with_http_only(true)
        .with_name(SESSION_CONFIG.name.clone())
        .with_private(Key::from(SESSION_CONFIG.key.as_bytes()))
        .with_same_site(SameSite::Strict)
        .with_secure(SESSION_CONFIG.secure);

    let shell = move |options| shell_with_app(options, app_fn);

    let app = Router::new()
        .leptos_routes(&leptos_options.clone(), routes, {
            let leptos_options = leptos_options.clone();
            move || shell_with_app(leptos_options.clone(), app_fn)
        })
        .fallback(file_and_error_handler(shell))
        .layer(session_layer)
        .with_state(leptos_options);

    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();

    redis_conn.await.unwrap().unwrap();
}
