// Image types — mirrors openai-python types/image.py + images_response.py

use serde::{Deserialize, Serialize};

/// Image quality level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageQuality {
    Standard,
    Hd,
    Low,
    Medium,
    High,
    Auto,
}

/// Image dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ImageSize {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "1024x1024")]
    S1024x1024,
    #[serde(rename = "1536x1024")]
    S1536x1024,
    #[serde(rename = "1024x1536")]
    S1024x1536,
    #[serde(rename = "256x256")]
    S256x256,
    #[serde(rename = "512x512")]
    S512x512,
    #[serde(rename = "1792x1024")]
    S1792x1024,
    #[serde(rename = "1024x1792")]
    S1024x1792,
}

/// Image style (dall-e-3 only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageStyle {
    Vivid,
    Natural,
}

/// Output format for generated images (GPT image models).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageOutputFormat {
    Png,
    Jpeg,
    Webp,
}

/// Response format for images.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageResponseFormat {
    Url,
    B64Json,
}

/// Background transparency for generated images (GPT image models).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageBackground {
    Transparent,
    Opaque,
    Auto,
}

/// Content moderation level for image generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageModeration {
    Low,
    Auto,
}

// ── Request types ──

/// Request body for `POST /images/generations`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageGenerateRequest {
    /// Text description of desired image(s).
    pub prompt: String,

    /// Model to use (e.g. "dall-e-3", "gpt-image-1").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Number of images to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Image quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,

    /// Response format: url or b64_json.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Image style (dall-e-3 only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ImageStyle>,

    /// End user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Output format (GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageOutputFormat>,

    /// Compression level 0–100 for webp/jpeg output (GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i64>,

    /// Background transparency (GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ImageBackground>,

    /// Content moderation level (GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<ImageModeration>,

    /// Number of partial images for streaming (0–3, GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_images: Option<i64>,

    /// Whether to stream the image generation (GPT image models only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl ImageGenerateRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            n: None,
            quality: None,
            response_format: None,
            size: None,
            style: None,
            user: None,
            output_format: None,
            output_compression: None,
            background: None,
            moderation: None,
            partial_images: None,
            stream: None,
        }
    }
}

/// Parameters for image edit (multipart upload).
#[derive(Debug)]
pub struct ImageEditParams {
    pub image: Vec<u8>,
    pub image_filename: String,
    pub prompt: String,
    pub model: Option<String>,
    pub mask: Option<(Vec<u8>, String)>,
    pub n: Option<i64>,
    pub size: Option<ImageSize>,
}

impl ImageEditParams {
    pub fn new(
        image: Vec<u8>,
        image_filename: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            image,
            image_filename: image_filename.into(),
            prompt: prompt.into(),
            model: None,
            mask: None,
            n: None,
            size: None,
        }
    }
}

/// Parameters for image variation (multipart upload).
#[derive(Debug)]
pub struct ImageVariationParams {
    pub image: Vec<u8>,
    pub image_filename: String,
    pub model: Option<String>,
    pub n: Option<i64>,
    pub size: Option<ImageSize>,
}

impl ImageVariationParams {
    pub fn new(image: Vec<u8>, image_filename: impl Into<String>) -> Self {
        Self {
            image,
            image_filename: image_filename.into(),
            model: None,
            n: None,
            size: None,
        }
    }
}

// ── Response types ──

/// A single generated image.
#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    /// Base64-encoded image data.
    #[serde(default)]
    pub b64_json: Option<String>,

    /// Revised prompt (dall-e-3).
    #[serde(default)]
    pub revised_prompt: Option<String>,

    /// URL of the generated image.
    #[serde(default)]
    pub url: Option<String>,
}

/// Response from image generation/edit/variation endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct ImagesResponse {
    pub created: i64,
    #[serde(default)]
    pub data: Option<Vec<Image>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_image_generate_request() {
        let req = ImageGenerateRequest::new("A cute baby sea otter");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["prompt"], "A cute baby sea otter");
        assert!(json.get("model").is_none());
    }

    #[test]
    fn test_serialize_image_generate_gpt_image_fields() {
        let mut req = ImageGenerateRequest::new("A futuristic city");
        req.model = Some("gpt-image-1".into());
        req.output_format = Some(ImageOutputFormat::Webp);
        req.output_compression = Some(80);
        req.background = Some(ImageBackground::Transparent);
        req.moderation = Some(ImageModeration::Low);
        req.partial_images = Some(2);
        req.stream = Some(true);

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["output_format"], "webp");
        assert_eq!(json["output_compression"], 80);
        assert_eq!(json["background"], "transparent");
        assert_eq!(json["moderation"], "low");
        assert_eq!(json["partial_images"], 2);
        assert_eq!(json["stream"], true);
    }

    #[test]
    fn test_deserialize_images_response_url() {
        let json = r#"{
            "created": 1589478378,
            "data": [
                {"url": "https://example.com/image.png", "revised_prompt": "A cute baby sea otter floating"}
            ]
        }"#;
        let resp: ImagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.created, 1589478378);
        let data = resp.data.unwrap();
        assert_eq!(data.len(), 1);
        assert!(data[0].url.is_some());
        assert!(data[0].revised_prompt.is_some());
    }

    #[test]
    fn test_deserialize_images_response_b64() {
        let json = r#"{
            "created": 1589478378,
            "data": [
                {"b64_json": "iVBORw0KGgoAAAANSUhEUg=="}
            ]
        }"#;
        let resp: ImagesResponse = serde_json::from_str(json).unwrap();
        let data = resp.data.unwrap();
        assert!(data[0].b64_json.is_some());
    }
}
