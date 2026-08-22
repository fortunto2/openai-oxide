// Containers resource — managed sandboxes used by Code Interpreter.
//
// API reference: <https://platform.openai.com/docs/api-reference/containers>

use crate::client::OpenAI;
use crate::error::OpenAIError;
use crate::types::containers::{
    ContainerCreateParams, ContainerCreateResponse, ContainerListParams, ContainerListResponse,
    ContainerRetrieveResponse, FileCreateParams, FileCreateResponse, FileListParams,
    FileListResponse, FileRetrieveResponse,
};

/// Access container endpoints.
///
/// API reference: <https://platform.openai.com/docs/api-reference/containers>
pub struct Containers<'a> {
    client: &'a OpenAI,
}

impl<'a> Containers<'a> {
    pub(crate) fn new(client: &'a OpenAI) -> Self {
        Self { client }
    }

    /// Files inside one container.
    ///
    /// ```ignore
    /// let files = client.containers().files("cntr_abc").list().await?;
    /// ```
    pub fn files(&self, container_id: &str) -> ContainerFiles<'a> {
        ContainerFiles {
            client: self.client,
            container_id: container_id.to_string(),
        }
    }

    /// Create a container.
    ///
    /// `POST /containers`
    pub async fn create(
        &self,
        params: &ContainerCreateParams,
    ) -> Result<ContainerCreateResponse, OpenAIError> {
        self.client.post("/containers", params).await
    }

    /// List containers.
    ///
    /// `GET /containers`
    pub async fn list(&self) -> Result<ContainerListResponse, OpenAIError> {
        self.client.get("/containers").await
    }

    /// List containers with pagination parameters.
    ///
    /// `GET /containers`
    pub async fn list_page(
        &self,
        params: &ContainerListParams,
    ) -> Result<ContainerListResponse, OpenAIError> {
        self.client
            .get_with_query("/containers", &container_list_query(params))
            .await
    }

    /// Retrieve a container.
    ///
    /// `GET /containers/{container_id}`
    pub async fn retrieve(
        &self,
        container_id: &str,
    ) -> Result<ContainerRetrieveResponse, OpenAIError> {
        self.client
            .get(&format!("/containers/{container_id}"))
            .await
    }

    /// Delete a container.
    ///
    /// `DELETE /containers/{container_id}`
    ///
    /// The spec documents no schema for the 200 body, so it comes back raw.
    pub async fn delete(&self, container_id: &str) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .delete(&format!("/containers/{container_id}"))
            .await
    }
}

/// File endpoints scoped to a single container.
pub struct ContainerFiles<'a> {
    client: &'a OpenAI,
    container_id: String,
}

impl ContainerFiles<'_> {
    /// Add a file to the container by referencing an already-uploaded file.
    ///
    /// `POST /containers/{container_id}/files`
    pub async fn create(
        &self,
        params: &FileCreateParams,
    ) -> Result<FileCreateResponse, OpenAIError> {
        self.client
            .post(&format!("/containers/{}/files", self.container_id), params)
            .await
    }

    /// Upload raw file content into the container.
    ///
    /// `POST /containers/{container_id}/files` (multipart)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn create_from_bytes(
        &self,
        filename: impl Into<String>,
        data: Vec<u8>,
    ) -> Result<FileCreateResponse, OpenAIError> {
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(data).file_name(filename.into()),
        );
        self.client
            .post_multipart(&format!("/containers/{}/files", self.container_id), form)
            .await
    }

    /// Upload a file from a filesystem path into the container.
    ///
    /// `POST /containers/{container_id}/files` (multipart)
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn create_from_path(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<FileCreateResponse, OpenAIError> {
        let path = path.as_ref();
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
            .to_string();
        let data = tokio::fs::read(path).await.map_err(|e| {
            OpenAIError::InvalidArgument(format!("failed to read {}: {e}", path.display()))
        })?;
        self.create_from_bytes(filename, data).await
    }

    /// List files in the container.
    ///
    /// `GET /containers/{container_id}/files`
    pub async fn list(&self) -> Result<FileListResponse, OpenAIError> {
        self.client
            .get(&format!("/containers/{}/files", self.container_id))
            .await
    }

    /// List files in the container with pagination parameters.
    ///
    /// `GET /containers/{container_id}/files`
    pub async fn list_page(
        &self,
        params: &FileListParams,
    ) -> Result<FileListResponse, OpenAIError> {
        self.client
            .get_with_query(
                &format!("/containers/{}/files", self.container_id),
                &file_list_query(params),
            )
            .await
    }

    /// Retrieve one file's metadata.
    ///
    /// `GET /containers/{container_id}/files/{file_id}`
    pub async fn retrieve(&self, file_id: &str) -> Result<FileRetrieveResponse, OpenAIError> {
        self.client
            .get(&format!(
                "/containers/{}/files/{file_id}",
                self.container_id
            ))
            .await
    }

    /// Retrieve one file's content as bytes.
    ///
    /// `GET /containers/{container_id}/files/{file_id}/content`
    pub async fn content(&self, file_id: &str) -> Result<bytes::Bytes, OpenAIError> {
        self.client
            .get_raw(&format!(
                "/containers/{}/files/{file_id}/content",
                self.container_id
            ))
            .await
    }

    /// Delete a file from the container.
    ///
    /// `DELETE /containers/{container_id}/files/{file_id}`
    ///
    /// The spec documents no schema for the 200 body, so it comes back raw.
    pub async fn delete(&self, file_id: &str) -> Result<serde_json::Value, OpenAIError> {
        self.client
            .delete(&format!(
                "/containers/{}/files/{file_id}",
                self.container_id
            ))
            .await
    }
}

/// The generated params structs carry no `to_query`, so build it here.
fn container_list_query(params: &ContainerListParams) -> Vec<(String, String)> {
    let mut query = Vec::new();
    if let Some(after) = &params.after {
        query.push(("after".to_string(), after.clone()));
    }
    if let Some(limit) = params.limit {
        query.push(("limit".to_string(), limit.to_string()));
    }
    if let Some(name) = &params.name {
        query.push(("name".to_string(), name.clone()));
    }
    if let Some(order) = &params.order {
        query.push(("order".to_string(), order_value(order).to_string()));
    }
    query
}

fn file_list_query(params: &FileListParams) -> Vec<(String, String)> {
    let mut query = Vec::new();
    if let Some(after) = &params.after {
        query.push(("after".to_string(), after.clone()));
    }
    if let Some(limit) = params.limit {
        query.push(("limit".to_string(), limit.to_string()));
    }
    if let Some(order) = &params.order {
        query.push(("order".to_string(), file_order_value(order).to_string()));
    }
    query
}

fn order_value(order: &crate::types::containers::ContainerListParamsOrder) -> &'static str {
    use crate::types::containers::ContainerListParamsOrder as Order;
    match order {
        Order::Asc => "asc",
        Order::Desc => "desc",
        _ => "desc",
    }
}

fn file_order_value(order: &crate::types::containers::FileListParamsOrder) -> &'static str {
    use crate::types::containers::FileListParamsOrder as Order;
    match order {
        Order::Asc => "asc",
        Order::Desc => "desc",
        _ => "desc",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::containers::{ContainerListParamsOrder, FileListParamsOrder};

    #[test]
    fn container_query_skips_unset_fields() {
        let params = ContainerListParams {
            after: None,
            limit: None,
            name: None,
            order: None,
        };
        assert!(container_list_query(&params).is_empty());
    }

    #[test]
    fn container_query_carries_every_set_field() {
        let params = ContainerListParams {
            after: Some("cntr_abc".to_string()),
            limit: Some(20),
            name: Some("sandbox".to_string()),
            order: Some(ContainerListParamsOrder::Asc),
        };
        assert_eq!(
            container_list_query(&params),
            vec![
                ("after".to_string(), "cntr_abc".to_string()),
                ("limit".to_string(), "20".to_string()),
                ("name".to_string(), "sandbox".to_string()),
                ("order".to_string(), "asc".to_string()),
            ]
        );
    }

    #[test]
    fn file_query_carries_every_set_field() {
        let params = FileListParams {
            after: Some("cfile_abc".to_string()),
            limit: Some(5),
            order: Some(FileListParamsOrder::Desc),
        };
        assert_eq!(
            file_list_query(&params),
            vec![
                ("after".to_string(), "cfile_abc".to_string()),
                ("limit".to_string(), "5".to_string()),
                ("order".to_string(), "desc".to_string()),
            ]
        );
    }

    #[test]
    fn container_response_deserializes() {
        let json = r#"{
            "id": "cntr_682dfebaacac8198bbfe9c2474fb6f4a085685cbe3cb5863",
            "object": "container",
            "created_at": 1747844794,
            "status": "running",
            "expires_after": { "anchor": "last_active_at", "minutes": 20 },
            "name": "My Container"
        }"#;
        let container: ContainerCreateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(container.object, "container");
        assert_eq!(container.status, "running");
    }

    #[test]
    fn container_file_response_deserializes() {
        let json = r#"{
            "id": "cfile_682e0e8a43c88191a7978f477a09bdf5",
            "object": "container.file",
            "created_at": 1747848842,
            "bytes": 880,
            "container_id": "cntr_682e0e7318108198aa783fd921ff305e08e78805b9fdbb04",
            "path": "/mnt/data/88e12fa445d32636f190a0b33daed6cb-tsconfig.json",
            "source": "user"
        }"#;
        let file: FileCreateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            file.container_id,
            "cntr_682e0e7318108198aa783fd921ff305e08e78805b9fdbb04"
        );
        assert_eq!(file.bytes, 880);
        assert_eq!(file.source, "user");
    }
}
