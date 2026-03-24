// Videos resource (Sora) — video generation, editing, extension, remix.
//
// OpenAI guide: <https://developers.openai.com/api/docs/guides/video-generation>
// API reference: <https://platform.openai.com/docs/api-reference/videos>

use crate::client::OpenAI;
use crate::error::OpenAIError;

/// Access video generation endpoints (Sora 2 / Sora 2 Pro).
///
/// API reference: <https://platform.openai.com/docs/api-reference/videos>
pub struct Videos<'a> {
    client: &'a OpenAI,
}

impl<'a> Videos<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Create a video generation job.
    ///
    /// `POST /videos`
    pub async fn create(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/videos", body).await
    }

    /// List video generation jobs.
    ///
    /// `GET /videos`
    pub async fn list(&self) -> Result<serde_json::Value, OpenAIError> {
        self.client.get("/videos").await
    }

    /// Retrieve video metadata.
    ///
    /// `GET /videos/{video_id}`
    pub async fn retrieve(&self, video_id: &str) -> Result<serde_json::Value, OpenAIError> {
        self.client.get(&format!("/videos/{video_id}")).await
    }

    /// Delete a video.
    ///
    /// `DELETE /videos/{video_id}`
    pub async fn delete(&self, video_id: &str) -> Result<serde_json::Value, OpenAIError> {
        self.client.delete(&format!("/videos/{video_id}")).await
    }

    /// Download video content bytes.
    ///
    /// `GET /videos/{video_id}/content`
    pub async fn content(&self, video_id: &str) -> Result<bytes::Bytes, OpenAIError> {
        self.client
            .get_raw(&format!("/videos/{video_id}/content"))
            .await
    }

    /// Create a video edit (modify an existing video).
    ///
    /// `POST /videos/edits`
    pub async fn edit(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/videos/edits", body).await
    }

    /// Extend a video (generate continuation).
    ///
    /// `POST /videos/extensions`
    pub async fn extend(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/videos/extensions", body).await
    }

    /// Remix a video (re-generate with modifications).
    ///
    /// `POST /videos/{video_id}/remix`
    pub async fn remix(
        &self,
        video_id: &str,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .post(&format!("/videos/{video_id}/remix"), body)
            .await
    }

    /// Create a reusable video character.
    ///
    /// `POST /videos/characters`
    pub async fn create_character(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client.post("/videos/characters", body).await
    }

    /// Retrieve a video character.
    ///
    /// `GET /videos/characters/{character_id}`
    pub async fn retrieve_character(
        &self,
        character_id: &str,
    ) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .get(&format!("/videos/characters/{character_id}"))
            .await
    }
}
