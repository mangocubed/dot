use leptos::prelude::{ServerFnError, server};

#[cfg(feature = "server")]
const SESSION_KEY_LANGUAGE: &str = "language";

#[cfg(feature = "server")]
async fn extract_session() -> Result<tower_sessions::Session, ServerFnError> {
    Ok(leptos_axum::extract::<tower_sessions::Session>().await?)
}

#[server]
pub async fn get_language() -> Result<Option<String>, ServerFnError> {
    let session = extract_session().await?;

    Ok(session.get::<String>(SESSION_KEY_LANGUAGE).await?)
}

#[server]
pub async fn set_language(value: String) -> Result<(), ServerFnError> {
    let session = extract_session().await?;

    Ok(session.insert(SESSION_KEY_LANGUAGE, value).await?)
}
