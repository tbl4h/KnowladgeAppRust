use iced::{
    executor, theme, window, Application, Command, Element, Length, Settings, Size,
};
use iced::widget::{column, container, row};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::Local;

// Deklaracja modułów
mod ui;
mod lm_studio_client;

// Importy z modułów
use ui::{create_sidebar, create_chat_area, create_save_dialog};
use lm_studio_client::{LMStudioClient, Message as LMMessage};

// Główna struktura aplikacji
#[derive(Debug)]
pub struct ChatApp {
    messages: Vec<ChatMessage>,
    input_value: String,
    current_conversation_name: String,
    saved_conversations: Vec<SavedConversation>,
    show_save_dialog: bool,
    save_name_input: String,
    lm_client: LMStudioClient,
}

impl Default for ChatApp {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input_value: String::new(),
            current_conversation_name: "Nowa rozmowa".to_string(),
            saved_conversations: Vec::new(),
            show_save_dialog: false,
            save_name_input: String::new(),
            lm_client: LMStudioClient::new(),
        }
    }
}

// Struktura dla wiadomości w czacie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
}

// Struktura dla zapisanych rozmów
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConversation {
    pub name: String,
    pub messages: Vec<ChatMessage>,
}

// Enum dla komunikatów w aplikacji
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    SendMessage,
    NewConversation,
    LoadConversation(usize),
    DeleteConversation(usize),
    ShowSaveDialog,
    HideSaveDialog,
    SaveNameChanged(String),
    ConfirmSave,
    ClearChat,
    MessageReceived(Result<String, String>),
}

impl Application for ChatApp {
    type Message = Message;
    type Theme = theme::Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut app = ChatApp::default();
        
        // Wczytaj zapisane rozmowy
        app.load_conversations();
        
        (app, Command::none())
    }

    fn title(&self) -> String {
        "Chat z LM Studio".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
                Command::none()
            }
            Message::SendMessage => {
                if !self.input_value.trim().is_empty() {
                    let user_message = ChatMessage {
                        content: self.input_value.clone(),
                        is_user: true,
                        timestamp: Local::now().format("%H:%M").to_string(),
                    };
                    
                    self.messages.push(user_message);
                    let input_content = self.input_value.clone();
                    self.input_value.clear();
                    
                    // Przygotuj historię wiadomości dla LM Studio
                    let mut history = Vec::new();
                    for msg in &self.messages {
                        history.push(LMMessage {
                            role: if msg.is_user { "user" } else { "assistant" }.to_string(),
                            content: msg.content.clone(),
                        });
                    }
                    
                    // Wyślij wiadomość do LM Studio
                    let client = self.lm_client.clone();
                    return Command::perform(
                        async move {
                            match client.send_message("local-model", history) {
                                Ok(response) => Ok(response),
                                Err(e) => Err(format!("Błąd komunikacji z LM Studio: {}", e)),
                            }
                        },
                        Message::MessageReceived,
                    );
                }
                Command::none()
            }
            Message::MessageReceived(result) => {
                let content = match result {
                    Ok(response) => response,
                    Err(error) => error,
                };
                
                let ai_message = ChatMessage {
                    content,
                    is_user: false,
                    timestamp: Local::now().format("%H:%M").to_string(),
                };
                self.messages.push(ai_message);
                Command::none()
            }
            Message::NewConversation => {
                self.messages.clear();
                self.current_conversation_name = "Nowa rozmowa".to_string();
                Command::none()
            }
            Message::LoadConversation(index) => {
                if let Some(conversation) = self.saved_conversations.get(index) {
                    self.messages = conversation.messages.clone();
                    self.current_conversation_name = conversation.name.clone();
                }
                Command::none()
            }
            Message::DeleteConversation(index) => {
                if index < self.saved_conversations.len() {
                    self.saved_conversations.remove(index);
                    self.save_conversations();
                }
                Command::none()
            }
            Message::ShowSaveDialog => {
                self.show_save_dialog = true;
                self.save_name_input = self.current_conversation_name.clone();
                Command::none()
            }
            Message::HideSaveDialog => {
                self.show_save_dialog = false;
                self.save_name_input.clear();
                Command::none()
            }
            Message::SaveNameChanged(name) => {
                self.save_name_input = name;
                Command::none()
            }
            Message::ConfirmSave => {
                if !self.save_name_input.trim().is_empty() && !self.messages.is_empty() {
                    let conversation = SavedConversation {
                        name: self.save_name_input.clone(),
                        messages: self.messages.clone(),
                    };
                    
                    // Sprawdź czy rozmowa o tej nazwie już istnieje
                    if let Some(existing_index) = self.saved_conversations
                        .iter()
                        .position(|c| c.name == conversation.name) {
                        self.saved_conversations[existing_index] = conversation;
                    } else {
                        self.saved_conversations.push(conversation);
                    }
                    
                    self.current_conversation_name = self.save_name_input.clone();
                    self.save_conversations();
                    self.show_save_dialog = false;
                    self.save_name_input.clear();
                }
                Command::none()
            }
            Message::ClearChat => {
                self.messages.clear();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let main_content = row![
            create_sidebar(self),
            create_chat_area(self)
        ]
        .spacing(0);

        if self.show_save_dialog {
            container(
                column![
                    main_content,
                    create_save_dialog(self)
                ]
            )
            .into()
        } else {
            main_content.into()
        }
    }

    fn theme(&self) -> theme::Theme {
        theme::Theme::Light
    }
}

impl ChatApp {
    fn save_conversations(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.saved_conversations) {
            let _ = fs::write("conversations.json", json);
        }
    }

    fn load_conversations(&mut self) {
        if Path::new("conversations.json").exists() {
            if let Ok(content) = fs::read_to_string("conversations.json") {
                if let Ok(conversations) = serde_json::from_str(&content) {
                    self.saved_conversations = conversations;
                }
            }
        }
    }
}

fn main() -> iced::Result {
    ChatApp::run(Settings {
        window: window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(800.0, 600.0)),
            ..Default::default()
        },
        default_text_size: iced::Pixels(14.0),
        ..Default::default()
    })
}