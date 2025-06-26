use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use reqwest::Url;

#[derive(Clone, Debug)]
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

#[derive(Deserialize)]
struct ModelEntry {
     pub id: String,
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

    pub fn list_models(&self) -> Result<Vec<String>, reqwest::Error> {
        let url = Url::parse("http://localhost:1234/v1/models").unwrap();
        let resp: serde_json::Value = self.client.get(url).send()?.json()?;
        let ids = resp["data"]
            .as_array().unwrap()
            .iter()
            .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(String::from))
            .collect();
        Ok(ids)
    }

    /// Wysyła synchronicznie historię czatu do LMStudio i zwraca odpowiedź
    pub fn send_message(
        &self,
        model: &str,
        history: Vec<Message>,
    ) -> Result<String, reqwest::Error> {
        let body = CompletionsRequest {
            model: model.to_string(),
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

        Ok(resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }
    /// Sprawdza aktualnie załadowany model
    pub fn get_loaded_model(&self) -> Result<Option<String>, reqwest::Error> {
        let resp: serde_json::Value = self
            .client
            .get("http://localhost:1234/v1/models/loaded")
            .send()?
            .json()?;

        Ok(resp.get("model").and_then(|v| v.as_str()).map(String::from))
    }

    /// Wymusza przeładowanie modelu przez zatrzymanie i ponowne uruchomienie
    pub fn force_reload_model(&self, model: &str) -> Result<(), reqwest::Error> {
        // Spróbuj zatrzymać serwer modelu
        let _ = self.client
            .post("http://localhost:1234/v1/server/stop")
            .send();

        // Poczekaj chwilę
        std::thread::sleep(Duration::from_millis(1000));

        // Wyślij żądanie z nowym modelem - to powinno go załadować na GPU
        let test_body = CompletionsRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "test".to_string(),
            }],
            temperature: 0.1,
            max_tokens: 1,
            stream: false,
        };

        // To żądanie spowoduje załadowanie modelu na GPU
        let _ = self
            .client
            .post("http://localhost:1234/v1/chat/completions")
            .header("Content-Type", "application/json")
            .json(&test_body)
            .send()?;

        Ok(())
    }

}