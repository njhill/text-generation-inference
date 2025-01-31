/// Text Generation Inference Webserver
mod batcher;
mod db;
pub mod server;
mod validation;

use batcher::{Batcher, InferResponse};
use db::{Db, Entry};
use serde::{Deserialize, Serialize};
use validation::Validation;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GenerateParameters {
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_k")]
    pub top_k: i32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default = "default_do_sample")]
    pub do_sample: bool,
    #[serde(default = "default_max_new_tokens")]
    pub max_new_tokens: u32,
}

fn default_temperature() -> f32 {
    1.0
}

fn default_top_k() -> i32 {
    0
}

fn default_top_p() -> f32 {
    1.0
}

fn default_do_sample() -> bool {
    false
}

fn default_max_new_tokens() -> u32 {
    20
}

fn default_parameters() -> GenerateParameters {
    GenerateParameters {
        temperature: default_temperature(),
        top_k: default_top_k(),
        top_p: default_top_p(),
        do_sample: default_do_sample(),
        max_new_tokens: default_max_new_tokens(),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GenerateRequest {
    pub inputs: String,
    #[serde(default = "default_parameters")]
    pub parameters: GenerateParameters,
}

#[derive(Serialize)]
pub(crate) struct GeneratedText {
    pub generated_text: String,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    pub error: String,
}
