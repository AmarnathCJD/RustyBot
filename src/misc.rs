// Purpose: Miscellaneous functions that don't fit anywhere else.
use grammers_client::types::{Chat, InputMessage, Message};
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
            let mut username = args[1];
            if let Ok(_id) = args[1].parse::<i64>() {
                message.reply("Finding by ID is not supported yet.").await?;
                return Ok(());
            }
            // trim @ from username
            username = username.trim_start_matches("@");
            let u = client.resolve_username(username).await;
            match u {
                Err(e) => {
                    message
                        .reply(InputMessage::html(format!(
                            "<code>{}</code>",
                            e.to_string()
                        )))
                        .await?;
                    return Ok(());
                }
                Ok(u) => {
                    if u.is_none() {
                        message
                            .reply(InputMessage::html(format!(
                                "<code>Username {} not found.</code>",
                                username
                            )))
                            .await?;
                        return Ok(());
                    }
                    chat = u.unwrap();
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
        let user_full = client.invoke(&req).await?;
        info.push_str("<b>User Info</b>\n");

        match user_full {
            tl::enums::users::UserFull::Full(user_full) => {
                match user_full.users[0].clone() {
                    tl::enums::User::User(u) => {
                        if u.first_name.to_owned().unwrap_or("".to_string()) != "" {
                            info.push_str(&format!(
                                "<code>First Name:</code> <b>{}</b>\n",
                                u.first_name.unwrap_or("".to_string())
                            ));
                        }
                        if u.last_name.to_owned().unwrap_or("".to_string()) != "" {
                            info.push_str(&format!(
                                "<code>Last Name:</code> <b>{}</b>\n",
                                u.last_name.unwrap_or("".to_string())
                            ));
                        }
                        info.push_str(&format!("<code>ID:</code> <b>{}</b>\n", chat.id()));
                        info.push_str(&format!(
                            "<code>Username:</code> <b>@{}</b>\n",
                            chat.username().unwrap_or("None")
                        ));
                        info.push_str(&format!(
                            "<code>Is Bot:</code> <b>{}</b>\n",
                            u.bot.to_string(),
                        ));
                        info.push_str(&format!(
                            "<code>Is Premium:</code> <b>{}</b>\n",
                            u.premium.to_string(),
                        ));
                    }

                    _ => {}
                }

                match user_full.full_user {
                    tl::enums::UserFull::Full(u) => {
                        if u.about.to_owned().unwrap_or("".to_string()) != "" {
                            info.push_str(&format!(
                                "\n<code>About:</code> <i>{}</i>\n\n",
                                u.about.unwrap_or("None".to_string())
                            ));
                        }
                    }
                }

                info.push_str(&format!(
                    "<code>Common Chats:</code> <b>{}</b>\n",
                    0 //user_full.common_chats_count.unwrap_or(0)
                ));
                info.push_str(&format!(
                    "<code>Gbanned:</code> <b>{}</b>\n",
                    "N/I"
                ));
            }
        }
    } else if let Chat::Channel(_) = chat {
        info.push_str("<b>Channel Info</b>\n");
        let input_channel = tl::enums::InputChannel::Channel(tl::types::InputChannel {
            channel_id: chat.id(),
            access_hash: chat.to_owned().pack().access_hash.unwrap(),
        });

        let req = tl::functions::channels::GetFullChannel {
            channel: input_channel.clone(),
        };

        let channel_full: tl::enums::messages::ChatFull = client.invoke(&req).await?;

        match channel_full {
            tl::enums::messages::ChatFull::Full(channel_full) => {
                match channel_full.chats[0].clone() {
                    tl::enums::Chat::Channel(c) => {
                        info.push_str(&format!(
                            "<code>Title:</code> <b>{}</b>\n",
                            c.title,
                        ));
                        info.push_str(&format!(
                            "<code>ID:</code> <b>{}</b>\n",
                            chat.id()
                        ));
                        if c.username.to_owned().unwrap_or("".to_string()) != "" {
                            info.push_str(&format!(
                                "<code>Username:</code> <b>@{}</b>\n",
                                c.username.unwrap_or("None".to_string())
                            ));
                        }
                        info.push_str(&format!(
                            "<code>Call Active:</code> <b>{}</b>\n",
                            c.call_active.to_string()
                        ));

                        info.push_str(&format!(
                            "<code>Members:</code> <b>{}</b>\n",
                            c.participants_count.unwrap_or(0)
                        ));

                        info.push_str(&format!(
                            "<code>Created:</code> <b>{}</b>\n",
                            c.date
                        ));
                    }
                    _ => {}
                }
                match channel_full.full_chat {
                    tl::enums::ChatFull::Full(c) => {
                        if c.about.to_owned() != "" {
                            info.push_str(&format!(
                                "\n<code>About:</code> <i>{}</i>\n\n",
                                c.about
                            ));
                        }
                    }
                    _ => {}
                }
            }
        } 
    } else if let Chat::Group(_) = chat {
        info.push_str("<b>Group Info</b>\n");
        let input_channel = tl::enums::InputChannel::Channel(tl::types::InputChannel {
            channel_id: chat.id(),
            access_hash: chat.to_owned().pack().access_hash.unwrap(),
        });

        let req = tl::functions::channels::GetFullChannel {
            channel: input_channel.clone(),
        };

        let channel_full: tl::enums::messages::ChatFull = client.invoke(&req).await?;
        match channel_full {
            tl::enums::messages::ChatFull::Full(channel_full) => {
                match channel_full.chats[0].clone() {
                    tl::enums::Chat::Channel(c) => {
                        info.push_str(&format!(
                            "<code>Title:</code> <b>{}</b>\n",
                            c.title,
                        ));
                        info.push_str(&format!(
                            "<code>ID:</code> <b>{}</b>\n",
                            chat.id()
                        ));
                        if c.username.to_owned().unwrap_or("".to_string()) != "" {
                            info.push_str(&format!(
                                "<code>Username:</code> <b>@{}</b>\n",
                                c.username.unwrap_or("None".to_string())
                            ));
                        }
                        info.push_str(&format!(
                            "<code>Call Active:</code> <b>{}</b>\n",
                            c.call_active.to_string()
                        ));
                        let date = chrono::NaiveDateTime::from_timestamp_opt(c.date as i64, 0).unwrap();
                        let now = chrono::Utc::now().naive_utc();
                        
                        let duration = now.signed_duration_since(date);

                        info.push_str(&format!(
                            "<code>Created:</code> <b>{}</b>\n",
                            duration.num_days().to_string() + " days ago"
                        ));
                    }
                    _ => {}
                }
                match channel_full.full_chat {
                    tl::enums::ChatFull::Full(c) => {
                        match c.participants {
                            tl::enums::ChatParticipants::Participants(p) => {
                                info.push_str(&format!(
                                    "<code>Members:</code> <b>{}</b>\n",
                                    p.participants.len()
                                ));
                            }
                            _ => {}
                        }
                        if c.about.to_owned() != "" {
                            info.push_str(&format!(
                                "\n<code>About:</code> <i>{}</i>\n\n",
                                c.about
                            ));
                        }
                    }

                    tl::enums::ChatFull::ChannelFull(c) => {
                        info.push_str(&format!(
                            "<code>Members:</code> <b>{}</b>\n",
                            c.participants_count.unwrap_or(0)
                        ));
                        if c.about.to_owned() != "" {
                            info.push_str(&format!(
                                "\n<code>About:</code> <i>{}</i>\n\n",
                                c.about
                            ));
                        }
                        
                        info.push_str(&format!(
                            "<b>A({})|K({})|B({})</b>\n",
                            c.admins_count.unwrap_or(0),
                            c.kicked_count.unwrap_or(0),
                            c.banned_count.unwrap_or(0)
                        ));
                    }
                }
            }
        } 
    }

    message.reply(InputMessage::html(info)).await?;

    Ok(())
}




pub async fn handle_paste(_client: Client, message: grammers_client::types::Message) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_paste = message.text().to_string(); // Need to make a mutable copy here

    if message.reply_to_message_id().unwrap_or(0) != 0 {
        let reply = message.get_reply().await?;
        to_paste = reply.expect("Meh").text().to_string(); // Make a mutable copy here as well
    }

    let msg = message.reply("Pasting...").await?;
    let client = reqwest::Client::new();
    
    let json_data = serde_json::json!({
        "content": to_paste,
    });
    
    let req = client.post("https://nekobin.com/api/documents")
        .json(&json_data)
        .timeout(std::time::Duration::from_secs(5))
        .send().await?;
    
    let response_json: serde_json::Value = req.json().await?;
    let key = response_json["result"]["key"].as_str().unwrap();
    let url = format!("https://nekobin.com/{}", key);

    msg.edit(InputMessage::html(format!("<b>Paste to <a href=\"{}\">[Nekobin]</a></b>", url))).await?;
    Ok(())
}
        
