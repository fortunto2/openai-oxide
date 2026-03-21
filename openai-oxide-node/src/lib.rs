#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use openai_oxide::OpenAI;
use openai_oxide::types::responses::ResponseCreateRequest;

#[napi]
pub struct Client {
    inner: OpenAI,
}

#[napi]
impl Client {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let inner = OpenAI::from_env().map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { inner })
    }

    #[napi]
    pub async fn create_response(&self, model: String, input: String) -> Result<String> {
        let req = ResponseCreateRequest::new(&model).input(input.as_str());
        let res = self.inner.responses().create(req).await.map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(res.output_text())
    }
}
