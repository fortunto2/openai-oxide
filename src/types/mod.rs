pub mod audio;
pub mod batch;
pub mod beta;
pub mod chat;
pub mod common;
pub mod embedding;
pub mod file;
pub mod fine_tuning;
pub mod image;
pub mod model;
pub mod moderation;
pub mod realtime;
pub mod responses;
pub mod upload;

// New domains — re-exported from openai-types
pub mod completion {
    pub use openai_types::completion::*;
}
pub mod containers {
    pub use openai_types::containers::*;
}
pub mod conversations {
    pub use openai_types::conversations::*;
}
pub mod evals {
    pub use openai_types::evals::*;
}
pub mod graders {
    pub use openai_types::graders::*;
}
pub mod shared {
    pub use openai_types::shared::*;
}
pub mod skills {
    pub use openai_types::skills::*;
}
pub mod vector_stores {
    pub use openai_types::vector_stores::*;
}
pub mod video {
    pub use openai_types::video::*;
}
pub mod webhooks_types {
    pub use openai_types::webhooks::*;
}
pub mod websocket_types {
    pub use openai_types::websocket::*;
}
