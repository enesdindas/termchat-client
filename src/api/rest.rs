use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::models::*;

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    message: String,
}

pub struct RestClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl RestClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token.as_deref().unwrap_or(""))
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let body: ApiResponse<T> = resp.json().await?;
        if let Some(err) = body.error {
            return Err(anyhow!("{}: {}", err.code, err.message));
        }
        body.data.ok_or_else(|| anyhow!("empty response"))
    }

    async fn post<B: serde::Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
        auth: bool,
    ) -> Result<T> {
        let mut req = self
            .client
            .post(format!("{}{}", self.base_url, path))
            .json(body);
        if auth {
            req = req.header("Authorization", self.auth_header());
        }
        let resp = req.send().await?;
        let api: ApiResponse<T> = resp.json().await?;
        if let Some(err) = api.error {
            return Err(anyhow!("{}: {}", err.code, err.message));
        }
        api.data.ok_or_else(|| anyhow!("empty response"))
    }

    async fn delete<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let resp = self
            .client
            .delete(format!("{}{}", self.base_url, path))
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        let api: ApiResponse<T> = resp.json().await?;
        if let Some(err) = api.error {
            return Err(anyhow!("{}: {}", err.code, err.message));
        }
        api.data.ok_or_else(|| anyhow!("empty response"))
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<User> {
        self.post(
            "/auth/register",
            &RegisterRequest {
                username: username.to_string(),
                password: password.to_string(),
            },
            false,
        )
        .await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginData> {
        self.post(
            "/auth/login",
            &LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            },
            false,
        )
        .await
    }

    pub async fn me(&self) -> Result<User> {
        self.get("/api/me").await
    }

    pub async fn list_channels(&self) -> Result<Vec<Channel>> {
        self.get("/api/channels").await
    }

    pub async fn create_channel(
        &self,
        name: &str,
        description: &str,
        is_private: bool,
    ) -> Result<Channel> {
        self.post(
            "/api/channels",
            &serde_json::json!({
                "name": name,
                "description": description,
                "is_private": is_private,
            }),
            true,
        )
        .await
    }

    pub async fn join_channel(&self, channel_id: i64) -> Result<Value> {
        self.post(
            &format!("/api/channels/{}/join", channel_id),
            &serde_json::json!({}),
            true,
        )
        .await
    }

    pub async fn list_members(&self, channel_id: i64) -> Result<Vec<ChannelMember>> {
        self.get(&format!("/api/channels/{}/members", channel_id))
            .await
    }

    pub async fn add_member(&self, channel_id: i64, user_id: i64) -> Result<Value> {
        self.post(
            &format!("/api/channels/{}/members", channel_id),
            &serde_json::json!({ "user_id": user_id }),
            true,
        )
        .await
    }

    pub async fn remove_member(&self, channel_id: i64, user_id: i64) -> Result<Value> {
        self.delete(&format!("/api/channels/{}/members/{}", channel_id, user_id))
            .await
    }

    pub async fn get_channel_messages(
        &self,
        channel_id: i64,
        before_id: Option<i64>,
    ) -> Result<Vec<Message>> {
        let mut path = format!("/api/channels/{}/messages", channel_id);
        if let Some(before) = before_id {
            path.push_str(&format!("?before={}&limit=50", before));
        }
        self.get(&path).await
    }

    pub async fn get_dm_history(
        &self,
        partner_id: i64,
        before_id: Option<i64>,
    ) -> Result<Vec<DirectMessage>> {
        let mut path = format!("/api/dm/{}", partner_id);
        if let Some(before) = before_id {
            path.push_str(&format!("?before={}&limit=50", before));
        }
        self.get(&path).await
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        self.get("/api/users").await
    }
}
