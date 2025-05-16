pub mod config {
    use std::sync::LazyLock;

    use serde::{Deserialize, Serialize};

    pub use leptos::config::{LeptosOptions, get_configuration as leptos_config};

    pub static SESSION_CONFIG: LazyLock<SessionConfig> = LazyLock::new(|| extract_from_env("SESSION_"));

    pub fn load_config() {
        use env_logger::{Env, init_from_env};

        let _ = dotenvy::dotenv();

        init_from_env(Env::new().default_filter_or("info"));
    }

    pub fn extract_from_env<'a, T>(prefix: &str) -> T
    where
        T: Deserialize<'a> + Serialize + Default,
    {
        use figment::Figment;
        use figment::providers::{Env, Serialized};

        Figment::from(Serialized::defaults(T::default()))
            .merge(Env::prefixed(prefix))
            .extract()
            .unwrap()
    }

    #[derive(Deserialize, Serialize)]
    pub struct SessionConfig {
        pub domain: String,
        pub key: String,
        pub name: String,
        pub redis_url: String,
        pub secure: bool,
    }

    impl Default for SessionConfig {
        fn default() -> Self {
            let db_number = if cfg!(test) { "11" } else { "1" };

            Self {
                domain: "localhost".to_owned(),
                key: "abcdefghijklmnopqrestuvvwxyz0123456789ABCDEFGHIJKLMNOPQRESTUVVWX".to_owned(),
                name: "_session".to_owned(),
                redis_url: format!("redis://127.0.0.1:6379/{db_number}"),
                secure: false,
            }
        }
    }
}

use leptos::prelude::{
    AutoReload, ElementChild, GlobalAttributes, HydrationScripts, IntoMaybeErased, IntoView, LeptosOptions, view,
};

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

pub async fn leptos_serve<IV>(app_fn: fn() -> IV) -> anyhow::Result<()>
where
    IV: IntoView + 'static,
{
    use axum::Router;
    use cookie::{Key, SameSite};
    use fred::prelude::{ClientLike, Config, Pool};
    use leptos::config::get_configuration;
    use leptos_axum::{LeptosRoutes, file_and_error_handler, generate_route_list};
    use time::Duration;
    use tokio::net::TcpListener;
    use tower_sessions::{Expiry, SessionManagerLayer};
    use tower_sessions_redis_store::RedisStore;

    use crate::server::config::SESSION_CONFIG;

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

    let app = Router::new()
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
