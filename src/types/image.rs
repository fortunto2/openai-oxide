// Image types — mirrors openai-python types/image.py + images_response.py

use crate::openai_enum;
use serde::{Deserialize, Serialize};

openai_enum! {
    /// Image quality level.
    pub enum ImageQuality {
        Standard = "standard",
        Hd = "hd",
        Low = "low",
        Medium = "medium",
        High = "high",
        Auto = "auto",
    }
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

openai_enum! {
    /// Image style (dall-e-3 only).
    pub enum ImageStyle {
        Vivid = "vivid",
        Natural = "natural",
    }
}

openai_enum! {
    /// Output format for generated images (GPT image models).
    pub enum ImageOutputFormat {
        Png = "png",
        Jpeg = "jpeg",
        Webp = "webp",
    }
}

openai_enum! {
    /// Response format for images.
    pub enum ImageResponseFormat {
        Url = "url",
        B64Json = "b64_json",
    }
}

openai_enum! {
    /// Background transparency for generated images (GPT image models).
    pub enum ImageBackground {
        Transparent = "transparent",
        Opaque = "opaque",
        Auto = "auto",
    }
}

openai_enum! {
    /// Content moderation level for image generation.
    pub enum ImageModeration {
        Low = "low",
        Auto = "auto",
    }
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

    /// Quality level (standard or hd for dall-e-3; low/medium/high/auto for gpt-image-1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<ImageQuality>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Response format (url or b64_json).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// Style (vivid or natural) — dall-e-3 only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ImageStyle>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    // GPT-image-1 specific fields
    /// Output format (png, jpeg, webp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<ImageOutputFormat>,

    /// Output compression quality (0–100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_compression: Option<i64>,

    /// Background style (transparent, opaque, auto).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<ImageBackground>,

    /// Moderation level (low, auto).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<ImageModeration>,
}

impl ImageGenerateRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            n: None,
            quality: None,
            size: None,
            response_format: None,
            style: None,
            user: None,
            output_format: None,
            output_compression: None,
            background: None,
            moderation: None,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn n(mut self, n: i64) -> Self {
        self.n = Some(n);
        self
    }

    pub fn quality(mut self, quality: ImageQuality) -> Self {
        self.quality = Some(quality);
        self
    }

    pub fn size(mut self, size: ImageSize) -> Self {
        self.size = Some(size);
        self
    }

    pub fn response_format(mut self, response_format: ImageResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn style(mut self, style: ImageStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn output_format(mut self, output_format: ImageOutputFormat) -> Self {
        self.output_format = Some(output_format);
        self
    }

    pub fn output_compression(mut self, output_compression: i64) -> Self {
        self.output_compression = Some(output_compression);
        self
    }

    pub fn background(mut self, background: ImageBackground) -> Self {
        self.background = Some(background);
        self
    }

    pub fn moderation(mut self, moderation: ImageModeration) -> Self {
        self.moderation = Some(moderation);
        self
    }
}

/// Request body for `POST /images/edits`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageEditRequest {
    /// Image to edit (multpart file).
    #[serde(skip_serializing)]
    pub image: Vec<u8>,
    #[serde(skip_serializing)]
    pub image_filename: String,

    /// Prompt describing the edit.
    pub prompt: String,

    /// Additional image for reference (optional).
    #[serde(skip_serializing)]
    pub mask: Option<Vec<u8>>,
    #[serde(skip_serializing)]
    pub mask_filename: Option<String>,

    /// Model to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Number of images to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl ImageEditRequest {
    pub fn new(
        image: Vec<u8>,
        image_filename: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            image,
            image_filename: image_filename.into(),
            prompt: prompt.into(),
            mask: None,
            mask_filename: None,
            model: None,
            n: None,
            size: None,
            response_format: None,
            user: None,
        }
    }
}

/// Backward compatibility alias.
pub type ImageEditParams = ImageEditRequest;

/// Request body for `POST /images/variations`.
#[derive(Debug, Clone, Serialize)]
pub struct ImageVariationRequest {
    /// Image to vary (multipart file).
    #[serde(skip_serializing)]
    pub image: Vec<u8>,
    #[serde(skip_serializing)]
    pub image_filename: String,

    /// Number of images to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,

    /// Image dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,

    /// Response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ImageResponseFormat>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl ImageVariationRequest {
    pub fn new(image: Vec<u8>, image_filename: impl Into<String>) -> Self {
        Self {
            image,
            image_filename: image_filename.into(),
            n: None,
            size: None,
            response_format: None,
            user: None,
        }
    }
}

/// Backward compatibility alias.
pub type ImageVariationParams = ImageVariationRequest;

// ── Response types ──

/// A single generated image.
#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    /// The URL of the generated image (if response_format is url).
    #[serde(default)]
    pub url: Option<String>,
    /// The base64-encoded JSON of the generated image (if response_format is b64_json).
    #[serde(default)]
    pub b64_json: Option<String>,
    /// The prompt that was used to generate the image (dall-e-3 only).
    #[serde(default)]
    pub revised_prompt: Option<String>,
}

impl Image {
    /// Decode the base64 image data and save it to a file.
    #[cfg(feature = "images")]
    pub fn save(&self, path: &std::path::Path) -> Result<(), crate::error::OpenAIError> {
        use std::io::Write;

        let b64 = self.b64_json.as_ref().ok_or_else(|| {
            crate::error::OpenAIError::InvalidArgument("No base64 data available".into())
        })?;
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64)
            .map_err(|e| {
                crate::error::OpenAIError::InvalidArgument(format!(
                    "Failed to decode base64: {}",
                    e
                ))
            })?;
        let mut file = std::fs::File::create(path).map_err(|e| {
            crate::error::OpenAIError::InvalidArgument(format!("Failed to create file: {}", e))
        })?;
        file.write_all(&bytes).map_err(|e| {
            crate::error::OpenAIError::InvalidArgument(format!("Failed to write file: {}", e))
        })?;
        Ok(())
    }
}

/// Response from image generation endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct ImagesResponse {
    pub created: i64,
    pub data: Vec<Image>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_image_generate() {
        let req = ImageGenerateRequest::new("A cute cat")
            .model("dall-e-3")
            .n(1);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["prompt"], "A cute cat");
        assert_eq!(json["model"], "dall-e-3");
        assert_eq!(json["n"], 1);
    }

    #[test]
    fn test_serialize_image_generate_gpt_image_fields() {
        let req = ImageGenerateRequest::new("A dog")
            .model("gpt-image-1")
            .quality(ImageQuality::High)
            .output_format(ImageOutputFormat::Webp)
            .background(ImageBackground::Transparent);
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-image-1");
        assert_eq!(json["quality"], "high");
        assert_eq!(json["output_format"], "webp");
        assert_eq!(json["background"], "transparent");
    }

    #[test]
    fn test_deserialize_images_response() {
        let json = r#"{
            "created": 1699012949,
            "data": [
                {"url": "https://example.com/image1.png"},
                {"url": "https://example.com/image2.png"}
            ]
        }"#;
        let resp: ImagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert!(resp.data[0].url.is_some());
    }

    #[test]
    fn test_deserialize_images_response_with_b64() {
        let json = r#"{
            "created": 1699012949,
            "data": [{"b64_json": "iVBORw0KGgo="}]
        }"#;
        let resp: ImagesResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data[0].b64_json.is_some());
    }
}
