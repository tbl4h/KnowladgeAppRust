use iced::{
    widget::{
        button, column, container, row, scrollable, text, text_input,
        Space, Tooltip,
    },
    alignment::{Horizontal},
    Element,
    Length,
    Color,
    Background,
    Border,
};
use crate::{ChatApp,  Message}; // Importuj potrzebne typy z głównego modułu

// Funkcje pomocnicze do tworzenia UI
pub fn create_sidebar(app: &ChatApp) -> Element<Message> {
    let mut sidebar_content = column![
        container(
            text("Rozmowy")
                .size(18)
                .style(Color::WHITE)
        )
        .padding(15)
        .style(container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            ..Default::default()
        }),
        
        container(
            button("+ Nowa rozmowa")
                .width(Length::Fill)
                .on_press(Message::NewConversation)
        )
        .padding(10),
    ]
    .spacing(5);

    // Dodaj zapisane rozmowy
    for (index, conversation) in app.saved_conversations.iter().enumerate() {
        sidebar_content = sidebar_content.push(
            container(
                row!(
                button(text(&conversation.name).size(14)) // Użyj conversation.name
                    .width(Length::Fill)
                    .on_press(Message::LoadConversation(index)),
                button(text("Usuń"))
                    .width(Length::Shrink)
                    .padding([0, 5]).on_press(Message::DeleteConversation(index)),
            ))
                .padding([0, 10])
        );
    }

    container(sidebar_content)
        .width(250)
        .height(Length::Fill)
        .style(container::Appearance {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            border: Border::with_radius(1.0),
            ..Default::default()
        })
        .into()
}

pub fn create_chat_area(app: &ChatApp) -> Element<Message> {
    let header = container(
        row![
            text(&app.current_conversation_name)
                .size(16)
                .width(Length::Fill),
            
            Tooltip::new(
                button(
                    container(text("").size(14))
                        .width(Length::Fixed(50.0)) // Przykład stałej szerokości
                        .height(Length::Fixed(30.0))
                        .style(container::Appearance {
                            background: Some(Background::Color(Color::from_rgb(0.0, 0.5, 1.0))),
                            ..Default::default()
                        })
                )
                .on_press(Message::ShowSaveDialog),
                "Zapisz rozmowę",
                iced::widget::tooltip::Position::Bottom
            ),
            
            Space::with_width(10),
            
            Tooltip::new(
                button(
                    container(text("").size(14))
                        .width(Length::Fixed(50.0)) // Przykład stałej szerokości
                        .height(Length::Fixed(30.0))
                        .style(container::Appearance {
                            background: Some(Background::Color(Color::from_rgb(1.0, 0.5, 0.0))),
                            ..Default::default()
                        })
                )
                .on_press(Message::ClearChat),
                "Wyczyść czat",
                iced::widget::tooltip::Position::Bottom
            ),
        ]
        .align_items(iced::Alignment::Center)
    )
    .padding(15)
    .style(container::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
        border: Border::with_radius(1.0),
        ..Default::default()
    });

    let messages_area = create_messages_view(app);
    let input_area = create_input_area(app);

    container(
        column![
            header,
            messages_area,
            input_area
        ]
        .spacing(0)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn create_messages_view(app: &ChatApp) -> Element<Message> {
    let mut messages_column = column![].spacing(10).padding(15);

    for message in &app.messages {
        let message_content = if message.is_user {
            // Wiadomość użytkownika - po prawej stronie
            row![
                Space::with_width(Length::FillPortion(1)),
                container(
                    column![
                        text(&message.content)
                            .size(14),
                        text(&message.timestamp)
                            .size(10)
                            .style(Color::from_rgb(0.6, 0.6, 0.6))
                    ]
                    .spacing(2)
                )
                .padding(12)
                .style(container::Appearance {
                    background: Some(Background::Color(Color::from_rgb(0.0, 0.5, 1.0))),
                    text_color: Some(Color::WHITE),
                    border: Border::with_radius(12),
                    ..Default::default()
                })
                .width(Length::FillPortion(3))
            ]
        } else {
            // Wiadomość AI - po lewej stronie
            row![
                container(
                    column![
                        text(&message.content)
                            .size(14),
                        text(&message.timestamp)
                            .size(10)
                            .style(Color::from_rgb(0.6, 0.6, 0.6))
                    ]
                    .spacing(2)
                )
                .padding(12)
                .style(container::Appearance {
                    background: Some(Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                    border: Border::with_radius(12),
                    ..Default::default()
                })
                .width(Length::FillPortion(3)),
                Space::with_width(Length::FillPortion(1))
            ]
        };
        
        messages_column = messages_column.push(message_content);
    }

    container(
        scrollable(messages_column)
            .height(Length::Fill)
    )
    .height(Length::Fill)
    .into()
}

pub fn create_input_area(app: &ChatApp) -> Element<Message> {
    container(
        row![
            text_input("Napisz wiadomość...", &app.input_value)
                .on_input(Message::InputChanged)
                .on_submit(Message::SendMessage)
                .padding(12)
                .width(Length::Fill),
            
            button("Wyślij")
                .on_press(Message::SendMessage)
                .padding([12, 20])
        ]
        .spacing(10)
        .align_items(iced::Alignment::Center)
    )
    .padding(15)
    .style(container::Appearance {
        background: Some(Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
        border: Border::with_radius(1.0),
        ..Default::default()
    })
    .into()
}

pub fn create_save_dialog(app: &ChatApp) -> Element<Message> {
    container(
        container(
            column![
                text("Zapisz rozmowę")
                    .size(18)
                    .horizontal_alignment(Horizontal::Center),
                
                Space::with_height(20),
                
                text("Nazwa rozmowy:"),
                text_input("Wpisz nazwę...", &app.save_name_input)
                    .on_input(Message::SaveNameChanged)
                    .on_submit(Message::ConfirmSave)
                    .padding(10),
                
                Space::with_height(20),
                
                row![
                    button("Anuluj")
                        .on_press(Message::HideSaveDialog)
                        ,
                    
                    Space::with_width(10),
                    
                    button("Zapisz")
                        .on_press(Message::ConfirmSave)
                ]
                .align_items(iced::Alignment::Center)
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center)
            .padding(30)
            .width(400)
        )
        .style(container::Appearance {
            background: Some(Background::Color(Color::WHITE)),
            border: Border::with_radius(10),
            ..Default::default()
        })
    )
    .center_x()
    .center_y()
    .width(Length::Fill)
    .height(Length::Fill)
    .style(container::Appearance {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        ..Default::default()
    })
    .into()
}