use leptos::prelude::{ServerFnError, server};

#[cfg(feature = "server")]
use super::{SESSION_KEY_LANGUAGE, extract_language, extract_session};

#[server]
pub async fn get_language() -> Result<Option<String>, ServerFnError> {
    Ok(Some(extract_language().await?.to_string()))
}

#[server]
pub async fn set_language(value: String) -> Result<(), ServerFnError> {
    let session = extract_session().await?;

    Ok(session
        .insert(SESSION_KEY_LANGUAGE, value.parse::<unic_langid::LanguageIdentifier>()?)
        .await?)
}
