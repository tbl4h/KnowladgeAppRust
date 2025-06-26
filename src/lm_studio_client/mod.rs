use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;


#[derive(Clone)]
pub struct LMStudioClient {
    client: Client,
}

#[derive(Serialize,Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// Struktury odpowiadające ciału żądania i odpowiedzi
#[derive(Serialize)]
struct CompletionsRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
    stream: bool,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: Message,
}

#[derive(Deserialize)]
struct CompletionsResponse {
    choices: Vec<ChatChoice>,
}

impl LMStudioClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(1200))
            .build()
            .expect("Nie udało się zbudować klienta HTTP");

        LMStudioClient {
            client
        }
    }

    /// Wysyła synchronicznie historię czatu do LMStudio i zwraca odpowiedź
    pub fn send_message(
        &self,
        history: Vec<Message>,
    ) -> Result<String, reqwest::Error> {
        let body = CompletionsRequest {
            model: "devstral-small-2505".to_string(),
            messages: history,
            temperature: 0.7,
            max_tokens: -1,
            stream: false,
        };

        let resp: CompletionsResponse = self
            .client
            .post("http://localhost:1234/v1/chat/completions")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?
            .error_for_status()?
            .json()?;

        // Pobieramy treść pierwszego wyboru
        Ok(resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }
}