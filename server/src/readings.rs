use std::{convert::Infallible, time::Duration};

use axum::{
    Json, Router,
    extract::{Query, State},
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
    routing::get,
};
use axum_security::cookie::CookieSession;
use futures_util::stream::{self, Stream, StreamExt};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::Db;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::{auth::User, error::AppResult, sensors::from_centi};

#[derive(Debug, toasty::Model)]
pub struct Reading {
    #[key]
    pub id: String,
    pub taken_at: i64,
    pub temp_c_centi: i64,
    pub humidity_centi: i64,
    pub moisture_centi: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadingJson {
    pub id: String,
    pub taken_at: Timestamp,
    pub temp_c: f64,
    pub humidity: f64,
    pub moisture: f64,
}

impl From<Reading> for ReadingJson {
    fn from(r: Reading) -> Self {
        Self {
            id: r.id,
            taken_at: Timestamp::from_second(r.taken_at).unwrap_or_else(|_| Timestamp::now()),
            temp_c: from_centi(r.temp_c_centi),
            humidity: from_centi(r.humidity_centi),
            moisture: from_centi(r.moisture_centi),
        }
    }
}

#[derive(Clone)]
pub struct ReadingsState {
    pub db: Db,
    pub tx: broadcast::Sender<ReadingJson>,
}

pub fn routes(state: ReadingsState) -> Router {
    Router::new()
        .route("/api/readings", get(list))
        .route("/api/readings/stream", get(stream))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub limit: Option<u32>,
}

async fn list(
    _session: CookieSession<User>,
    State(state): State<ReadingsState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<ReadingJson>>> {
    let mut db = state.db.clone();
    let from = q.from.unwrap_or(0);
    let to = q.to.unwrap_or(i64::MAX);
    let limit = q.limit.unwrap_or(500).min(5000) as usize;

    let rows: Vec<Reading> = toasty::query!(Reading filter .taken_at >= #from and .taken_at <= #to)
        .exec(&mut db)
        .await?;

    let mut out: Vec<ReadingJson> = rows.into_iter().map(Into::into).collect();
    out.sort_by_key(|r| r.taken_at);
    if out.len() > limit {
        let start = out.len() - limit;
        out.drain(..start);
    }
    Ok(Json(out))
}

async fn stream(
    _session: CookieSession<User>,
    State(state): State<ReadingsState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let s = BroadcastStream::new(rx).filter_map(|res| async move {
        match res {
            Ok(reading) => Event::default()
                .json_data(&reading)
                .ok()
                .map(Ok::<_, Infallible>),
            Err(_) => None,
        }
    });
    let stream = stream::once(async { Ok(Event::default().comment("open")) }).chain(s);
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
