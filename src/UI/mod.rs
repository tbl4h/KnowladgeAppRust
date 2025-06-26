// Deklaracja pliku zawierającego wszystkie funkcje UI dla tego modułu
pub mod chat_application_ui;

// Reeksportowanie funkcji publicznych z ChatApplicationUI dla łatwiejszego dostępu
pub use chat_application_ui::{create_sidebar, create_chat_area, create_save_dialog};

