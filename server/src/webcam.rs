use std::path::{Path, PathBuf};

use axum::{
    Json, Router,
    extract::{Path as AxumPath, State},
    routing::get,
};
use axum_security::cookie::CookieSession;
use jiff::civil::Date;
use serde::Serialize;
use tokio::fs;

use crate::{
    auth::User,
    error::{AppError, AppResult},
};

const PLAYLIST_FILENAME: &str = "live.m3u8";
const ARCHIVE_FILENAME: &str = "day.mp4";
const URL_PREFIX: &str = "/webcam";

#[derive(Clone)]
pub struct WebcamState {
    pub root: PathBuf,
}

pub fn routes(state: WebcamState) -> Router {
    Router::new()
        .route("/api/webcam/live", get(live))
        .route("/api/webcam/archive", get(archive_list))
        .route("/api/webcam/archive/{date}", get(archive_day))
        .with_state(state)
}

#[derive(Serialize)]
pub struct LiveResponse {
    pub url: Option<String>,
    pub date: Option<String>,
}

#[derive(Serialize)]
pub struct ArchiveEntry {
    pub date: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct ArchiveDayResponse {
    pub url: String,
}

async fn live(
    _session: CookieSession<User>,
    State(state): State<WebcamState>,
) -> AppResult<Json<LiveResponse>> {
    let dates = scan_date_dirs(&state.root).await?;
    for date in dates.into_iter().rev() {
        if fs::try_exists(state.root.join(&date).join(PLAYLIST_FILENAME))
            .await
            .unwrap_or(false)
        {
            return Ok(Json(LiveResponse {
                url: Some(format!("{URL_PREFIX}/{date}/{PLAYLIST_FILENAME}")),
                date: Some(date),
            }));
        }
    }
    Ok(Json(LiveResponse {
        url: None,
        date: None,
    }))
}

async fn archive_list(
    _session: CookieSession<User>,
    State(state): State<WebcamState>,
) -> AppResult<Json<Vec<ArchiveEntry>>> {
    let mut dates = scan_date_dirs(&state.root).await?;
    dates.sort();
    let mut out = Vec::new();
    for date in dates.into_iter().rev() {
        if fs::try_exists(state.root.join(&date).join(ARCHIVE_FILENAME))
            .await
            .unwrap_or(false)
        {
            out.push(ArchiveEntry {
                url: format!("{URL_PREFIX}/{date}/{ARCHIVE_FILENAME}"),
                date,
            });
        }
    }
    Ok(Json(out))
}

async fn archive_day(
    _session: CookieSession<User>,
    State(state): State<WebcamState>,
    AxumPath(date): AxumPath<String>,
) -> AppResult<Json<ArchiveDayResponse>> {
    let date = parse_date(&date)?;
    let path = state.root.join(&date).join(ARCHIVE_FILENAME);
    if !fs::try_exists(&path).await.unwrap_or(false) {
        return Err(AppError::NotFound);
    }
    Ok(Json(ArchiveDayResponse {
        url: format!("{URL_PREFIX}/{date}/{ARCHIVE_FILENAME}"),
    }))
}

/// Validate `YYYY-MM-DD` and return it canonicalized. Rejects path traversal
/// and any input that doesn't round-trip through `jiff`.
fn parse_date(s: &str) -> AppResult<String> {
    let parsed =
        Date::strptime("%Y-%m-%d", s).map_err(|_| AppError::BadRequest("invalid date".into()))?;
    let canonical = parsed.strftime("%Y-%m-%d").to_string();
    if canonical != s {
        return Err(AppError::BadRequest("invalid date".into()));
    }
    Ok(canonical)
}

/// Return all directory entries under `root` whose name parses as
/// `YYYY-MM-DD`. Sorted ascending. Missing root → empty list.
async fn scan_date_dirs(root: &Path) -> AppResult<Vec<String>> {
    let mut dir = match fs::read_dir(root).await {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(AppError::internal(format!("read webcam root: {e}"))),
    };
    let mut out = Vec::new();
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AppError::internal(format!("read webcam dir: {e}")))?
    {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if Date::strptime("%Y-%m-%d", &name).is_ok()
            && entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false)
        {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}
