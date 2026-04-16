use axum_security::cookie::{CookieSession, CookieStore, SessionId};
use jiff::Timestamp;
use toasty::Db;
use uuid::Uuid;

use crate::{
    auth::User,
    error::{AppError, AppResult},
};

#[derive(Debug, toasty::Model)]
pub struct Session {
    #[key]
    pub id: String,
    pub session_created_at: i64,
    pub user_id: String,
    pub subject: String,
    pub email: String,
    pub name: Option<String>,
    pub user_created_at: i64,
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

impl CookieStore for ToastySessionStore {
    type State = User;
    type Error = AppError;

    async fn store_session(&self, session: CookieSession<User>) -> AppResult<()> {
        let mut db = self.db.clone();
        let id = session.session_id.as_str().to_string();

        Session::filter_by_id(&id).delete().exec(&mut db).await?;

        Session::create()
            .id(id)
            .session_created_at(session.created_at as i64)
            .user_id(session.state.id.to_string())
            .subject(session.state.subject)
            .email(session.state.email)
            .name(session.state.name)
            .user_created_at(session.state.created_at.as_second())
            .exec(&mut db)
            .await?;

        Ok(())
    }

    async fn load_session(&self, id: &SessionId) -> AppResult<Option<CookieSession<User>>> {
        let mut db = self.db.clone();
        let Some(row) = Session::filter_by_id(id.as_str().to_string())
            .first()
            .exec(&mut db)
            .await?
        else {
            return Ok(None);
        };
        let user = User {
            id: Uuid::parse_str(&row.user_id)
                .map_err(|e| AppError::internal(format!("bad user_id: {e}")))?,
            subject: row.subject,
            email: row.email,
            name: row.name,
            created_at: Timestamp::from_second(row.user_created_at)
                .map_err(|e| AppError::internal(format!("bad timestamp: {e}")))?,
        };
        Ok(Some(CookieSession::new(
            id.clone(),
            row.session_created_at as u64,
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
        let cutoff = deadline as i64;
        toasty::query!(Session filter .session_created_at <= #cutoff)
            .delete()
            .exec(&mut db)
            .await?;
        Ok(())
    }
}
