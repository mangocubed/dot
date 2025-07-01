use std::sync::LazyLock;

use fluent_templates::{StaticLoader, static_loader};
use leptos::prelude::*;
use leptos_fluent::leptos_fluent;
use leptos_meta::provide_meta_context;

pub use leptos_fluent::{I18n, move_tr, tr};

#[cfg(feature = "server")]
pub use leptos_axum::{extract, redirect};

pub mod components;
pub mod forms;
pub mod icons;

mod server_functions;

use server_functions::{get_language, set_language};

#[cfg(feature = "server")]
use crate::constants::SESSION_KEY_LANGUAGE;

static_loader! {
    static DOT_TRANSLATIONS = {
        locales: "./locales",
        fallback_language: "en",
    };
}

pub fn use_i18n() -> I18n {
    use_context().unwrap()
}

#[cfg(feature = "server")]
pub async fn extract_language() -> Result<unic_langid::LanguageIdentifier, ServerFnError> {
    let session = extract_session().await?;

    Ok(session
        .get(SESSION_KEY_LANGUAGE)
        .await?
        .unwrap_or_else(|| unic_langid::langid!("en")))
}

#[cfg(feature = "server")]
pub async fn extract_session() -> Result<tower_sessions::Session, ServerFnError> {
    Ok(extract::<tower_sessions::Session>().await?)
}

#[cfg(feature = "hydrate")]
pub fn hydrate_body<IV>(app_fn: fn() -> IV)
where
    IV: IntoView + 'static,
{
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(app_fn)
}

#[cfg(feature = "server")]
pub async fn serve<IV>(app_fn: fn() -> IV) -> anyhow::Result<()>
where
    IV: IntoView + 'static,
{
    serve_with_axum_router(app_fn, axum::Router::new()).await
}

#[cfg(feature = "server")]
pub async fn serve_with_axum_router<IV>(
    app_fn: fn() -> IV,
    router: axum::Router<leptos::config::LeptosOptions>,
) -> anyhow::Result<()>
where
    IV: IntoView + 'static,
{
    use cookie::{Key, SameSite};
    use fred::prelude::{ClientLike, Config, Pool};
    use leptos::config::get_configuration;
    use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
    use time::Duration;
    use tokio::net::TcpListener;
    use tower_sessions::{Expiry, SessionManagerLayer};
    use tower_sessions_redis_store::RedisStore;

    use crate::config::SESSION_CONFIG;

    let leptos_options = get_configuration(None)?.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(app_fn);
    let redis_pool = Pool::new(Config::from_url(&SESSION_CONFIG.redis_url)?, None, None, None, 10)?;

    let redis_conn = redis_pool.connect();
    redis_pool.wait_for_connect().await?;

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

    let app = router
        .leptos_routes(&leptos_options.clone(), routes, {
            let leptos_options = leptos_options.clone();
            move || shell_with_app(leptos_options.clone(), app_fn)
        })
        .fallback(file_and_error_handler(shell))
        .layer(session_layer)
        .with_state(leptos_options);

    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, app.into_make_service()).await?;

    redis_conn.await.unwrap()?;

    Ok(())
}

#[cfg(feature = "server")]
fn shell_with_app<IV>(options: leptos::config::LeptosOptions, app_fn: fn() -> IV) -> impl IntoView
where
    IV: IntoView + 'static,
{
    use leptos::hydration::{AutoReload, HydrationScripts};
    use leptos_meta::{HashedStylesheet, MetaTags};

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=0"
                />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <MetaTags />
                <HashedStylesheet id="leptos" options=options />
            </head>
            <body>{app_fn()}</body>
        </html>
    }
}

#[component]
pub fn AppProvider(
    #[prop(optional)] loading_icon_src: Option<&'static str>,
    translations: &'static LazyLock<StaticLoader>,
    children: Children,
) -> impl IntoView {
    provide_meta_context();

    let is_done = RwSignal::new(false);

    Effect::new(move || is_done.set(true));

    view! {
        <I18nProvider translations=translations>
            <div>{children()}</div>

            <div class="loading-overlay" class:is-done=is_done>
                <figure>
                    <div class="loading-pulse"></div>
                    {loading_icon_src.map(|icon_src| view! { <img src=icon_src /> })}
                </figure>
            </div>
        </I18nProvider>
    }
}

#[component]
fn I18nProvider(translations: &'static LazyLock<StaticLoader>, children: Children) -> impl IntoView {
    leptos_fluent! {
        #[cfg(debug_assertions)]
        check_translations: "./src/**/*.rs",

        languages: "./locales/languages.yaml",
        locales: "./locales",
        sync_html_tag_dir: true,
        sync_html_tag_lang: true,
        translations: [DOT_TRANSLATIONS, translations],

        initial_language_from_server_function: get_language,
        set_language_to_server_function: set_language,

        children: children(),
    }
}
