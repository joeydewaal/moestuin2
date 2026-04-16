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
use uuid::Uuid;

use crate::{auth::User, error::AppResult};

fn serialize_from_centi<S: serde::Serializer>(v: &i64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(*v as f64 / 100.0)
}

#[derive(Debug, Clone, toasty::Model, Serialize)]
pub struct Reading {
    #[key]
    pub id: Uuid,
    pub taken_at: Timestamp,
    #[serde(rename = "temp_c", serialize_with = "serialize_from_centi")]
    pub temp_c_centi: i64,
    #[serde(rename = "humidity", serialize_with = "serialize_from_centi")]
    pub humidity_centi: i64,
    #[serde(rename = "moisture", serialize_with = "serialize_from_centi")]
    pub moisture_centi: i64,
}

#[derive(Clone)]
pub struct ReadingsState {
    pub db: Db,
    pub tx: broadcast::Sender<Reading>,
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
) -> AppResult<Json<Vec<Reading>>> {
    let mut db = state.db.clone();
    let from = q
        .from
        .map(Timestamp::from_second)
        .transpose()?
        .unwrap_or(Timestamp::UNIX_EPOCH);
    let to =
        q.to.map(Timestamp::from_second)
            .transpose()?
            .unwrap_or(Timestamp::MAX);
    let limit = q.limit.unwrap_or(500).min(5000) as usize;

    let rows: Vec<Reading> = toasty::query!(Reading filter .taken_at >= #from and .taken_at <= #to)
        .exec(&mut db)
        .await?;

    let mut out = rows;
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
