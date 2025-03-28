use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{config};

// Структуры для работы с Telegram API
#[derive(Debug, Serialize, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    chat: Chat,
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Debug, Serialize)]
struct SendMessage {
    chat_id: i64,
    text: String,
}

// Проверка отправлял ли пользователь exit, и если да, то остановить прогу
pub async fn check(len: usize) {
    let client = reqwest::Client::new();
    // Получаем обновления с обработкой ошибок
    match get_updates(&client, &format!("https://api.telegram.org/bot{}/", config::BOT_TOKEN), 0).await {
        Ok(updates) => {
            if (updates.len() != len) {
                if let Some(update) = updates.last() {  // <- Безопасный способ через .last()

                    if let Some(message) = &update.message {  // <- Также заимствуем message
                        if let Some(text) = &message.text {
                            if text == "exit" {
                                std::process::exit(0);  // Гарантированное завершение
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Ошибка при получении обновлений: {}", e);
        }
    }

    // Небольшая пауза между запросами
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}


// Возвращает последние отправленные в телеграмм сообщения
async fn get_updates(client: &reqwest::Client, api_url: &str, offset: i64) -> Result<Vec<Update>, reqwest::Error> {
    let response = client
        .get(format!("{}getUpdates", api_url))
        .query(&[("offset", offset), ("timeout", 30)])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let updates: Vec<Update> = serde_json::from_value(response["result"].clone()).unwrap_or_default();
    Ok(updates)
}

// отправка сообщения в тг
pub async fn bot_sender(text: String) {
    // Получаем токен бота и ID чата
    let bot_token = config::BOT_TOKEN;
    let chat_id = config::CHAT_ID;

    // Формируем URL для API Telegram
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    // Создаем HTTP клиент
    let client = reqwest::Client::new();

    // Отправляем POST запрос с параметрами
    let _response = client.post(&url)
        .json(&json!({
            "chat_id": chat_id,
            "text": text
        }))
        .send()
        .await;
}