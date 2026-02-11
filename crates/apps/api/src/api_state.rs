use crate::timeline::websocket::MediaPayload;
use app_state::{AppSettings, IngestSettings};
use axum::extract::FromRef;
use common_services::s2s_client::S2SClient;
use futures::future::poll_fn;
use open_clip_inference::TextEmbedder;
use sqlx::PgPool;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use tokio::sync::broadcast;

#[derive(Debug)]
struct LifoSemaphoreInner {
    permits: usize,
    waiters: VecDeque<Waker>,
}

#[derive(Debug)]
pub struct LifoSemaphore(Arc<Mutex<LifoSemaphoreInner>>);

impl LifoSemaphore {
    pub fn new(permits: usize) -> Self {
        Self(Arc::new(Mutex::new(LifoSemaphoreInner {
            permits,
            waiters: VecDeque::new(),
        })))
    }

    pub async fn acquire(&self) -> LifoPermit {
        poll_fn(|cx| {
            let mut inner = self.0.lock().unwrap();
            if inner.permits > 0 {
                inner.permits -= 1;
                return Poll::Ready(LifoPermit(self.0.clone()));
            }
            // Add current task's waker to the list of waiters
            inner.waiters.push_back(cx.waker().clone());
            Poll::Pending
        })
        .await
    }
}

pub struct LifoPermit(Arc<Mutex<LifoSemaphoreInner>>);

impl Drop for LifoPermit {
    fn drop(&mut self) {
        let mut inner = self.0.lock().unwrap();
        inner.permits += 1;
        // LIFO: Pop from the BACK of the queue to wake the most recent request
        if let Some(waker) = inner.waiters.pop_back() {
            waker.wake();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiContext {
    pub pool: PgPool,
    pub s2s_client: S2SClient,
    pub settings: AppSettings,
    pub timeline_broadcaster: broadcast::Sender<Arc<MediaPayload>>,
    pub embedder: Arc<TextEmbedder>,
    pub thumbnail_semaphore: Arc<LifoSemaphore>,
}

impl FromRef<ApiContext> for PgPool {
    fn from_ref(state: &ApiContext) -> Self {
        state.pool.clone()
    }
}

impl FromRef<ApiContext> for S2SClient {
    fn from_ref(state: &ApiContext) -> Self {
        state.s2s_client.clone()
    }
}

impl FromRef<ApiContext> for AppSettings {
    fn from_ref(state: &ApiContext) -> Self {
        state.settings.clone()
    }
}

impl FromRef<ApiContext> for IngestSettings {
    fn from_ref(state: &ApiContext) -> Self {
        state.settings.ingest.clone()
    }
}
