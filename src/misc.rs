// Purpose: Miscellaneous functions that don't fit anywhere else.

use grammers_client::types::{Chat, ChatMap, InputMessage, Message};
use grammers_client::Client;
use grammers_tl_types as tl;
use grammers_tl_types::functions::users::GetFullUser;

pub async fn get_info_handler(
    client: Client,
    message: Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut info = String::new();

    let chat: Chat;
    let args = message.text().split_whitespace().collect::<Vec<&str>>();
    if message.reply_to_message_id().unwrap_or(0) == 0 {
        if args.len() == 1 {
            chat = message.sender().unwrap();
        } else {
            let username = args[1];
            if let Ok(_id) = args[1].parse::<i64>() {
                message.reply("Finding by ID is not supported yet.").await?;
                return Ok(());
            }
            let u = client.resolve_username(username).await;
            match u {
                Ok(u) => {
                    chat = u.unwrap();
                }
                Err(e) => {
                    message
                        .reply(InputMessage::html(format!(
                            "<code>{}</code>",
                            e.to_string()
                        )))
                        .await?;
                    return Ok(());
                }
            }
        }
    } else {
        let reply = message.get_reply().await.unwrap();
        chat = reply.unwrap().sender().unwrap();
    }

    // chat or user info
    if let Chat::User(_) = chat {
        let input_user = tl::enums::InputUser::User(tl::types::InputUser {
            user_id: chat.id(),
            access_hash: chat.to_owned().pack().access_hash.unwrap(),
        });
        let req = GetFullUser {
            id: input_user.clone(),
        };
        let u = client.invoke(&req).await?;
        info.push_str("<b>User Info</b>\n");
        info.push_str(&format!("<code>Name:</code> <b>{}</b>\n", chat.name()));
        info.push_str(&format!("<code>ID:</code> <b>{}</b>\n", chat.id()));
        info.push_str(&format!(
            "<code>Username:</code> <b>@{}</b>\n",
            chat.username().unwrap_or("None")
        ));
    }

    message.reply(InputMessage::html(info)).await?;

    Ok(())
}
