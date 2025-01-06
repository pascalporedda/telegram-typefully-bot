use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use teloxide::types::Chat;
use time::OffsetDateTime;

pub struct Database {
    pool: Pool<Sqlite>,
}

pub struct UserPayload {
    pub telegram_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub telegram_id: i64,
    pub username: String,
    pub typefully_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    #[allow(unused)]
    pub created_at: OffsetDateTime,
}

impl User {
    pub async fn update_key(&self, db: &Database, api_key: &str) -> Result<()> {
        sqlx::query!(
            r#"UPDATE users SET typefully_api_key = ? WHERE telegram_id = ?"#,
            api_key,
            self.telegram_id
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn update_openai_api_key(&self, db: &Database, openai_api_key: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET openai_api_key = ?
            WHERE telegram_id = ?
            "#,
            openai_api_key,
            self.telegram_id
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }
}

pub const FREE_USAGE_LIMIT_SECONDS: i32 = 300;

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn get_user(&self, telegram_id: u64) -> Result<Option<User>> {
        let user_id = telegram_id as i64;
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT telegram_id, username, created_at, typefully_api_key, openai_api_key
            FROM users
            WHERE telegram_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_user(&self, user_payload: UserPayload) -> Result<User> {
        let now = OffsetDateTime::now_utc();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (telegram_id, username, created_at, typefully_api_key, openai_api_key)
            VALUES (?, ?, ?, NULL, NULL) RETURNING *
            "#,
            user_payload.telegram_id,
            user_payload.name,
            now,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_total_usage_seconds(&self, telegram_id: i64) -> Result<i32> {
        let total_seconds = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(duration_seconds), 0) 
            FROM voice_note_usage
            WHERE telegram_id = ?
            "#,
            telegram_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(total_seconds)
    }

    pub async fn add_usage(&self, telegram_id: i64, duration_seconds: i32) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query!(
            r#"
            INSERT INTO voice_note_usage (telegram_id, duration_seconds, created_at)
            VALUES (?, ?, ?)
            "#,
            telegram_id,
            duration_seconds,
            now,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn has_free_usage(&self, telegram_id: i64) -> Result<bool> {
        let total_usage = self.get_total_usage_seconds(telegram_id).await?;
        Ok(total_usage < FREE_USAGE_LIMIT_SECONDS)
    }

    pub async fn mark_user_deleted(&self, telegram_id: i64, total_usage: i32) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        sqlx::query!(
            r#"
            INSERT INTO deleted_users (telegram_id, total_usage_seconds, deleted_at)
            VALUES (?, ?, ?)
            "#,
            telegram_id,
            total_usage,
            now,
        )
        .execute(&self.pool)
        .await?;

        // Delete user but keep their usage records
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE telegram_id = ?
            "#,
            telegram_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

impl From<Chat> for UserPayload {
    fn from(chat: Chat) -> Self {
        Self {
            telegram_id: chat.id.0 as i64,
            name: chat.first_name().unwrap_or_default().to_string(),
        }
    }
}
