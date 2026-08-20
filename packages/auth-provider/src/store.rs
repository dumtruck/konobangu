use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use oxide_auth::primitives::grant::{Extensions, Grant, Value};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, QueryFilter, Set, Statement,
  TransactionTrait,
};
use sea_query::Iden;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use crate::{
  ProviderError,
  schema::{self, State, col, table},
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct StoredGrant {
  pub owner_id: String,
  pub client_id: String,
  pub redirect_uri: String,
  pub scope: String,
  pub until: i64,
  pub nonce: Option<String>,
  pub auth_time: i64,
  pub extensions: Vec<(String, bool, Option<String>)>,
}
impl StoredGrant {
  pub fn from_grant(grant: Grant) -> Result<Self, ProviderError> {
    let extensions: Vec<_> = grant
      .extensions
      .public()
      .map(|(key, value)| (key.to_owned(), false, value.map(str::to_owned)))
      .chain(grant.extensions.private().map(|(key, value)| (key.to_owned(), true, value.map(str::to_owned))))
      .collect();
    let identity = extensions
      .iter()
      .find(|(key, _, _)| key == "konobangu_oidc")
      .and_then(|(_, _, value)| value.as_ref())
      .ok_or(ProviderError::Protocol)?;
    let identity: OidcData = serde_json::from_str(identity).map_err(|_| ProviderError::Protocol)?;
    Ok(Self {
      owner_id: grant.owner_id,
      client_id: grant.client_id,
      redirect_uri: grant.redirect_uri.to_string(),
      scope: grant.scope.to_string(),
      until: grant.until.timestamp(),
      nonce: identity.nonce,
      auth_time: identity.auth_time,
      extensions,
    })
  }
  pub fn to_grant(&self) -> Result<Grant, ProviderError> {
    let mut extensions = Extensions::new();
    for (key, private, value) in &self.extensions {
      extensions.set_raw(
        key.clone(),
        if *private {
          Value::private(value.clone())
        } else {
          Value::public(value.clone())
        },
      );
    }
    Ok(Grant {
      owner_id: self.owner_id.clone(),
      client_id: self.client_id.clone(),
      redirect_uri: self.redirect_uri.parse().map_err(|_| ProviderError::Storage)?,
      scope: self.scope.parse().map_err(|_| ProviderError::Storage)?,
      until: chrono::DateTime::from_timestamp(self.until, 0).ok_or(ProviderError::Storage)?,
      extensions,
    })
  }
}
#[derive(Serialize, Deserialize)]
pub(crate) struct OidcData {
  pub nonce: Option<String>,
  pub auth_time: i64,
}

#[derive(Clone)]
pub(crate) struct Store {
  pub db: DatabaseConnection,
  pub issuer: String,
}
pub(crate) fn secret() -> String {
  format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple())
}
pub(crate) fn digest(secret: &str) -> String {
  URL_SAFE_NO_PAD.encode(Sha256::digest(secret.as_bytes()))
}
fn statement(sql: String, values: Vec<sea_orm::Value>) -> Statement {
  Statement::from_sql_and_values(DbBackend::Postgres, sql, values)
}

impl Store {
  fn key(&self, secret: &str) -> String {
    digest(&format!("{}\0{secret}", self.issuer))
  }
  pub async fn put<T: Serialize>(&self, kind: &str, data: &T, expires: i64) -> Result<String, ProviderError> {
    let token = secret();
    let tx = self.db.begin().await.map_err(|_| ProviderError::Storage)?;
    self.insert(&tx, &token, kind, data, None, expires).await?;
    self.prune(&tx).await?;
    tx.commit().await.map_err(|_| ProviderError::Storage)?;
    Ok(token)
  }
  async fn insert<T: Serialize>(
    &self,
    db: &DatabaseTransaction,
    token: &str,
    kind: &str,
    data: &T,
    family: Option<&str>,
    expires: i64,
  ) -> Result<(), ProviderError> {
    let payload = serde_json::to_value(data).map_err(|_| ProviderError::Storage)?;
    schema::ActiveModel {
      hash: Set(self.key(token)),
      kind: Set(kind.to_owned()),
      data: Set(payload),
      family: Set(family.map(str::to_owned)),
      expires_at: Set(expires),
      consumed: Set(false),
    }
    .insert(db)
    .await
    .map_err(|_| ProviderError::Storage)?;
    Ok(())
  }
  pub async fn take<T: DeserializeOwned>(&self, token: &str, kind: &str) -> Result<Option<T>, ProviderError> {
    let row = self
      .db
      .query_one_raw(statement(
        format!(
          "DELETE FROM {} WHERE {}=$1 AND {}=$2 RETURNING {},{}",
          table(),
          col(State::Hash),
          col(State::Kind),
          col(State::Data),
          col(State::ExpiresAt)
        ),
        vec![self.key(token).into(), kind.into()],
      ))
      .await
      .map_err(|_| ProviderError::Storage)?;
    let Some(row) = row else { return Ok(None) };
    let expires: i64 = row.try_get("", &State::ExpiresAt.to_string()).map_err(|_| ProviderError::Storage)?;
    if expires <= Utc::now().timestamp() {
      return Ok(None);
    }
    let value: serde_json::Value = row.try_get("", &State::Data.to_string()).map_err(|_| ProviderError::Storage)?;
    serde_json::from_value(value).map(Some).map_err(|_| ProviderError::Storage)
  }
  pub async fn exists(&self, token: &str, kind: &str) -> Result<bool, ProviderError> {
    Ok(
      schema::Entity::find_by_id(self.key(token))
        .filter(State::Kind.eq(kind))
        .filter(State::ExpiresAt.gt(Utc::now().timestamp()))
        .one(&self.db)
        .await
        .map_err(|_| ProviderError::Storage)?
        .is_some(),
    )
  }
  pub async fn issue_refresh(&self, grant: &StoredGrant, expires: i64) -> Result<String, ProviderError> {
    let token = secret();
    let family = uuid::Uuid::new_v4().to_string();
    let tx = self.db.begin().await.map_err(|_| ProviderError::Storage)?;
    let mut grant = grant.clone();
    grant.until = expires;
    self.insert(&tx, &family, "family", &serde_json::json!({}), None, expires).await?;
    self.insert(&tx, &token, "refresh", &grant, Some(&family), expires).await?;
    self.prune(&tx).await?;
    tx.commit().await.map_err(|_| ProviderError::Storage)?;
    Ok(token)
  }
  /// A consumed record is retained until its original expiry to detect reuse.
  pub async fn recover_refresh(&self, token: &str) -> Result<Option<StoredGrant>, ProviderError> {
    let tx = self.db.begin().await.map_err(|_| ProviderError::Storage)?;
    let result = self.lock_refresh(&tx, token).await?;
    tx.commit().await.map_err(|_| ProviderError::Storage)?;
    Ok(result.map(|(grant, _, _)| grant))
  }
  async fn lock_refresh(&self, tx: &DatabaseTransaction, token: &str) -> Result<Option<(StoredGrant, String, i64)>, ProviderError> {
    let row = tx
      .query_one_raw(statement(
        format!(
          "SELECT {} FROM {} WHERE {}=$1 AND {}='refresh'",
          col(State::Family),
          table(),
          col(State::Hash),
          col(State::Kind)
        ),
        vec![self.key(token).into()],
      ))
      .await
      .map_err(|_| ProviderError::Storage)?;
    let Some(row) = row else { return Ok(None) };
    let family: String = row.try_get("", &State::Family.to_string()).map_err(|_| ProviderError::Storage)?;
    let family_row = tx
      .query_one_raw(statement(
        format!(
          "SELECT {} FROM {} WHERE {}=$1 AND {}='family' FOR UPDATE",
          col(State::Consumed),
          table(),
          col(State::Hash),
          col(State::Kind)
        ),
        vec![self.key(&family).into()],
      ))
      .await
      .map_err(|_| ProviderError::Storage)?;
    let Some(family_row) = family_row else { return Ok(None) };
    if family_row
      .try_get::<bool>("", &State::Consumed.to_string())
      .map_err(|_| ProviderError::Storage)?
    {
      return Ok(None);
    }
    let row = tx
      .query_one_raw(statement(
        format!(
          "SELECT * FROM {} WHERE {}=$1 AND {}='refresh' FOR UPDATE",
          table(),
          col(State::Hash),
          col(State::Kind)
        ),
        vec![self.key(token).into()],
      ))
      .await
      .map_err(|_| ProviderError::Storage)?
      .ok_or(ProviderError::Storage)?;
    let consumed: bool = row.try_get("", &State::Consumed.to_string()).map_err(|_| ProviderError::Storage)?;
    let expires: i64 = row.try_get("", &State::ExpiresAt.to_string()).map_err(|_| ProviderError::Storage)?;
    if consumed {
      tx.execute_raw(statement(
        format!("UPDATE {} SET {}=true WHERE {}=$1", table(), col(State::Consumed), col(State::Hash)),
        vec![self.key(&family).into()],
      ))
      .await
      .map_err(|_| ProviderError::Storage)?;
      return Ok(None);
    }
    if expires <= Utc::now().timestamp() {
      return Ok(None);
    }
    let value: serde_json::Value = row.try_get("", &State::Data.to_string()).map_err(|_| ProviderError::Storage)?;
    Ok(Some((serde_json::from_value(value).map_err(|_| ProviderError::Storage)?, family, expires)))
  }
  pub async fn rotate_refresh(&self, old: &str, grant: &StoredGrant) -> Result<Option<String>, ProviderError> {
    let tx = self.db.begin().await.map_err(|_| ProviderError::Storage)?;
    let Some((previous, family, expires)) = self.lock_refresh(&tx, old).await? else {
      tx.commit().await.map_err(|_| ProviderError::Storage)?;
      return Ok(None);
    };
    if previous.client_id != grant.client_id
      || previous.owner_id != grant.owner_id
      || !grant
        .scope
        .split_whitespace()
        .all(|scope| previous.scope.split_whitespace().any(|allowed| scope == allowed))
    {
      return Err(ProviderError::Protocol);
    }
    tx.execute_raw(statement(
      format!("UPDATE {} SET {}=true WHERE {}=$1", table(), col(State::Consumed), col(State::Hash)),
      vec![self.key(old).into()],
    ))
    .await
    .map_err(|_| ProviderError::Storage)?;
    let next = secret();
    let mut grant = grant.clone();
    grant.until = expires;
    self.insert(&tx, &next, "refresh", &grant, Some(&family), expires).await?;
    self.prune(&tx).await?;
    tx.commit().await.map_err(|_| ProviderError::Storage)?;
    Ok(Some(next))
  }
  async fn prune(&self, tx: &DatabaseTransaction) -> Result<(), ProviderError> {
    tx.execute_raw(statement(
      format!(
        "DELETE FROM {} WHERE {} IN (SELECT {} FROM {} WHERE {}<$1 LIMIT 64)",
        table(),
        col(State::Hash),
        col(State::Hash),
        table(),
        col(State::ExpiresAt)
      ),
      vec![Utc::now().timestamp().into()],
    ))
    .await
    .map_err(|_| ProviderError::Storage)?;
    Ok(())
  }
}
