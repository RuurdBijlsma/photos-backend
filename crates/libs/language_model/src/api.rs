use async_stream::try_stream;
use base64::{engine::general_purpose, Engine as _};
use bon::bon;
use color_eyre::Result;
use futures_util::{Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;

#[derive(Error, Debug)]
pub enum LlamaError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error (status {status}): {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
}

pub type LlamaResult<T> = Result<T, LlamaError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<MessagePart>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MessagePart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    temperature: f32,
    top_p: f32,
    repetition_penalty: f32,
    presence_penalty: f32,
}

#[derive(Deserialize)]
struct ChatChunk {
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize)]
struct ChunkChoice {
    delta: ChunkDelta,
}

#[derive(Deserialize)]
struct ChunkDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
pub struct ChatFullResponse {
    pub choices: Vec<FullChoice>,
}

#[derive(Deserialize)]
pub struct FullChoice {
    pub message: FullMessage,
}

#[derive(Deserialize)]
pub struct FullMessage {
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ChatEvent {
    Content(String),
    Reasoning(String),
}

#[derive(Clone)]
pub struct LlamaClient {
    http: reqwest::Client,
    base_url: String,
    config: LlamaConfig,
}

#[derive(Clone)]
pub struct LlamaConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub repetition_penalty: f32,
    pub presence_penalty: f32,
}

#[bon]
impl LlamaClient {
    #[builder(start_fn = with_base_url)]
    #[must_use]
    pub fn new(
        #[builder(start_fn)] base_url: &str,
        temperature: Option<f32>,
        top_p: Option<f32>,
        repetition_penalty: Option<f32>,
        presence_penalty: Option<f32>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.to_string(),
            config: LlamaConfig {
                temperature: temperature.unwrap_or(0.7),
                top_p: top_p.unwrap_or(0.8),
                repetition_penalty: repetition_penalty.unwrap_or(1.0),
                presence_penalty: presence_penalty.unwrap_or(1.5),
            },
        }
    }

    const fn build_request(
        &self,
        model: String,
        messages: Vec<Message>,
        stream: bool,
    ) -> ChatRequest {
        ChatRequest {
            model,
            messages,
            stream,
            top_p: self.config.top_p,
            temperature: self.config.temperature,
            repetition_penalty: self.config.repetition_penalty,
            presence_penalty: self.config.presence_penalty,
        }
    }

    pub async fn full_request(
        &self,
        model: String,
        messages: Vec<Message>,
    ) -> LlamaResult<ChatFullResponse> {
        let req_body = self.build_request(model, messages, false);
        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self.http.post(url).json(&req_body).send().await?;

        if !response.status().is_success() {
            return Err(LlamaError::Api {
                status: response.status(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        Ok(response.json().await?)
    }

    pub async fn stream_request(
        &self,
        model: String,
        messages: Vec<Message>,
    ) -> LlamaResult<Pin<Box<dyn Stream<Item = LlamaResult<ChatEvent>> + Send>>> {
        let req_body = self.build_request(model, messages, true);
        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self.http.post(url).json(&req_body).send().await?;
        if !response.status().is_success() {
            return Err(LlamaError::Api {
                status: response.status(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        let stream_bytes = response
            .bytes_stream()
            .map_err(std::io::Error::other);
        let reader = StreamReader::new(stream_bytes);
        let mut lines = BufReader::new(reader).lines();
        Ok(Box::pin(try_stream! {
            while let Some(line) = lines.next_line().await.map_err(LlamaError::Io)? {
                let line = line.trim();
                if line.is_empty() || line == "data: [DONE]" { continue; }
                if let Some(data) = line.strip_prefix("data: ") {
                    let chunk = serde_json::from_str::<ChatChunk>(data).map_err(LlamaError::Json)?;
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(r) = &choice.delta.reasoning_content {
                            yield ChatEvent::Reasoning(r.clone());
                        }
                        if let Some(c) = &choice.delta.content {
                            yield ChatEvent::Content(c.clone());
                        }
                    }
                }
            }
        }))
    }
}

pub struct ChatResponseStream<'a> {
    inner: Pin<Box<dyn Stream<Item = LlamaResult<ChatEvent>> + Send>>,
    session: &'a mut ChatSession,
    accumulated_content: String,
}

impl Stream for ChatResponseStream<'_> {
    type Item = LlamaResult<ChatEvent>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = self.inner.poll_next_unpin(cx);
        match &result {
            Poll::Ready(Some(Ok(ChatEvent::Content(c)))) => {
                self.accumulated_content.push_str(c);
            }
            Poll::Ready(None) => {
                let content = std::mem::take(&mut self.accumulated_content);
                if !content.is_empty() {
                    self.session.push_text("assistant", content);
                }
            }
            _ => {}
        }
        result
    }
}

pub struct ChatSession {
    client: LlamaClient,
    model: String,
    messages: Vec<Message>,
}

#[bon]
impl ChatSession {
    #[builder(start_fn = with_client)]
    #[must_use]
    pub fn new(#[builder(start_fn)] client: LlamaClient, model: Option<String>) -> Self {
        Self {
            client,
            model: model.unwrap_or_default(),
            messages: Vec::new(),
        }
    }

    async fn prepare_user_message(
        &mut self,
        prompt: &str,
        images: &[impl AsRef<Path> + Sync],
    ) -> LlamaResult<()> {
        let mut parts = vec![MessagePart::Text {
            text: prompt.to_string(),
        }];
        for path in images {
            let bytes = fs::read(path).await?;
            let mime_type = infer::get(&bytes).map_or("image/jpeg", |kind| kind.mime_type());
            let b64 = general_purpose::STANDARD.encode(&bytes);
            parts.push(MessagePart::ImageUrl {
                image_url: ImageUrl {
                    url: format!("data:{mime_type};base64,{b64}"),
                },
            });
        }
        self.messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::Parts(parts),
        });
        Ok(())
    }

    pub fn push_text(&mut self, role: &str, text: String) {
        self.messages.push(Message {
            role: role.to_string(),
            content: MessageContent::Text(text),
        });
    }

    #[builder]
    pub async fn chat(
        &mut self,
        #[builder(start_fn)] prompt: &str,
        images: Option<&[&Path]>,
    ) -> LlamaResult<String> {
        self.prepare_user_message(prompt, images.unwrap_or_default())
            .await?;
        let response = self
            .client
            .full_request(self.model.clone(), self.messages.clone())
            .await?;
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        self.push_text("assistant", content.clone());
        Ok(content)
    }

    #[builder]
    pub async fn chat_stream<'a>(
        &'a mut self,
        #[builder(start_fn)] prompt: &str,
        images: Option<&[&Path]>,
    ) -> LlamaResult<ChatResponseStream<'a>> {
        self.prepare_user_message(prompt, images.unwrap_or_default())
            .await?;
        let inner = self
            .client
            .stream_request(self.model.clone(), self.messages.clone())
            .await?;
        Ok(ChatResponseStream {
            inner,
            session: self,
            accumulated_content: String::new(),
        })
    }

    pub fn reset(&mut self) {
        self.messages.clear();
    }
}