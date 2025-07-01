#[cfg(feature = "server")]
use std::future::Future;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
pub struct CursorPage<T> {
    pub end_cursor: Option<Uuid>,
    pub has_next_page: bool,
    pub nodes: Vec<T>,
}

impl<T> Default for CursorPage<T> {
    fn default() -> Self {
        Self {
            end_cursor: None,
            has_next_page: false,
            nodes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CursorPageParams {
    pub after: Option<Uuid>,
    pub first: u8,
}

impl Default for CursorPageParams {
    fn default() -> Self {
        Self { after: None, first: 10 }
    }
}

impl CursorPageParams {
    pub fn new(after: Option<Uuid>, first: u8) -> Self {
        Self { after, first }
    }
}

#[cfg(feature = "server")]
impl<T> CursorPage<T> {
    pub async fn new<CT, CF, RT, RF, QF>(
        cursor_page_params: &CursorPageParams,
        cursor_fn: CF,
        cursor_resource_fn: RF,
        query_fn: QF,
    ) -> CursorPage<T>
    where
        CF: Fn(&T) -> Uuid,
        CT: Future<Output = Option<T>>,
        RF: Fn(Uuid) -> CT,
        RT: Future<Output = Vec<T>>,
        QF: Fn(Option<T>, i64) -> RT,
    {
        let cursor_resource = if let Some(after) = cursor_page_params.after {
            cursor_resource_fn(after).await
        } else {
            None
        };
        let limit = cursor_page_params.first + 1;
        let mut nodes = query_fn(cursor_resource, limit.into()).await;

        let has_next_page = if nodes.len() > cursor_page_params.first as usize {
            nodes.remove(nodes.len() - 1);

            true
        } else {
            false
        };

        let end_cursor = nodes.last().map(|n| cursor_fn(n));

        Self {
            end_cursor,
            nodes,
            has_next_page,
        }
    }
}
