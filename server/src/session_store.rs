use axum_security::cookie::{CookieSession, CookieStore, SessionId};
use jiff::Timestamp;
use toasty::Db;
use uuid::Uuid;

use crate::{
    auth::User,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, toasty::Model)]
pub struct Session {
    #[key]
    pub id: String,
    pub session_created_at: Timestamp,
    #[index]
    pub user_id: Uuid,
    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<User>,
}

#[derive(Clone)]
pub struct ToastySessionStore {
    db: Db,
}

impl ToastySessionStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

fn ts_from_secs(secs: u64) -> AppResult<Timestamp> {
    Timestamp::from_second(secs as i64)
        .map_err(|e| AppError::internal(format!("bad timestamp: {e}")))
}

impl CookieStore for ToastySessionStore {
    type State = User;
    type Error = AppError;

    async fn store_session(&self, session: CookieSession<User>) -> AppResult<()> {
        let mut db = self.db.clone();
        let id = session.session_id.as_str().to_string();

        Session::filter_by_id(&id).delete().exec(&mut db).await?;

        Session::create()
            .id(id)
            .session_created_at(ts_from_secs(session.created_at)?)
            .user_id(session.state.id)
            .exec(&mut db)
            .await?;

        Ok(())
    }

    async fn load_session(&self, id: &SessionId) -> AppResult<Option<CookieSession<User>>> {
        let mut db = self.db.clone();
        let Some(row) = Session::filter_by_id(id.as_str().to_string())
            .include(Session::fields().user())
            .first()
            .exec(&mut db)
            .await?
        else {
            return Ok(None);
        };
        let user = row.user.get().clone();
        Ok(Some(CookieSession::new(
            id.clone(),
            row.session_created_at.as_second() as u64,
            user,
        )))
    }

    async fn remove_session(&self, id: &SessionId) -> AppResult<Option<CookieSession<User>>> {
        let existing = self.load_session(id).await?;
        if existing.is_some() {
            let mut db = self.db.clone();
            Session::filter_by_id(id.as_str().to_string())
                .delete()
                .exec(&mut db)
                .await?;
        }
        Ok(existing)
    }

    async fn remove_before(&self, deadline: u64) -> AppResult<()> {
        let mut db = self.db.clone();
        let cutoff = ts_from_secs(deadline)?;
        toasty::query!(Session filter .session_created_at <= #cutoff)
            .delete()
            .exec(&mut db)
            .await?;
        Ok(())
    }
}
