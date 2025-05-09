#[cfg(feature = "ssr")]
use leptos::prelude::{
    AutoReload, ElementChild, GlobalAttributes, HydrationScripts, IntoMaybeErased, IntoView, LeptosOptions, view,
};

pub mod icons;

#[cfg(feature = "hydrate")]
pub fn hydrate_body<IV>(app_fn: fn() -> IV)
where
    IV: leptos::prelude::IntoView + 'static,
{
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(app_fn)
}

pub mod config {
    pub use leptos::config::{LeptosOptions, get_configuration};
}

pub mod meta {
    pub use leptos_meta::{Title, provide_meta_context};
}

pub mod prelude {
    pub use leptos::prelude::{
        Children, ClassAttribute, CustomAttribute, ElementChild, IntoMaybeErased, IntoView, Signal, component, view,
    };
}

pub mod router {
    pub use leptos_router::StaticSegment;
    pub use leptos_router::components::{Route, Router, Routes};
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
    use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
    use tokio::net::TcpListener;

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(app_fn);

    let shell = move |options| shell_with_app(options, app_fn);

    let app = Router::new()
        .leptos_routes(&leptos_options.clone(), routes, {
            let leptos_options = leptos_options.clone();
            move || shell_with_app(leptos_options.clone(), app_fn)
        })
        .fallback(file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service()).await.unwrap();
}
