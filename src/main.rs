use grammers_client::{Client, Config, InitParams, Update, InputMessage};
use grammers_session::Session;
use grammers_tl_types::types as tl;
use grammers_tl_types::enums as enums;
use tokio::{runtime, task};
use reqwest;
use serde_json;

// use grammers_tl_types::types::MessageEntityBold;
// use grammers_tl_types::enums::MessageEntity; // If this import works, use it

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

async fn handle_update(client: Client, update: Update) -> Result {
    // println!("RustyBot -> Handling update: {:?}", update);
    match update {
        Update::NewMessage(message) if !message.outgoing() => {
            if message.text() == "/start" {
                handle_start_command(client, message).await?;
            } else if message.text() == "/ping" {
                handle_ping(client, message).await?;
            } else if message.text().to_string().starts_with("/paste") {
                handle_paste(client, message).await?;
            }
        }
        _ => {}
    }

    Ok(())
}

async fn is_chat_private(message: grammers_client::types::Message) -> bool {
    if let grammers_client::types::Chat::User(_) = message.chat() {
        return true;
    } else {
        return false;
    }
}

async fn handle_ping(client: Client, message: grammers_client::types::Message) -> Result {
    let chat = message.chat();
    let start_time = std::time::Instant::now();

    let msg = client.send_message(&chat, "Pong").await?;

    let end_time = std::time::Instant::now();
    let elapsed_time = end_time - start_time;
    let _text_msg = format!("Pong: {:?}!!", elapsed_time);
    let entities: Vec<enums::MessageEntity> = vec![
        enums::MessageEntity::Bold(tl::MessageEntityBold {
            offset: 0,
            length: 5, //_text_msg.len() as i32
        }),
        enums::MessageEntity::Code(tl::MessageEntityCode {
            offset: 6,
            length: format!("{:?}", elapsed_time).len() as i32,
        }), 
    ];
    client.edit_message(&chat, msg.id(), InputMessage::text(_text_msg).fmt_entities(entities)).await?;

    Ok(())
}

async fn handle_paste(client: Client, message: grammers_client::types::Message) -> Result {
    let chat = message.chat();
    let mut to_paste = message.text().to_string(); // Need to make a mutable copy here

    if message.reply_to_message_id().unwrap_or(0) != 0 {
        let reply = message.get_reply().await?;
        to_paste = reply.expect("Meh").text().to_string(); // Make a mutable copy here as well
    }

    let client = reqwest::blocking::Client::new();
    
    let json_data = serde_json::json!({
        "content": to_paste,
    });
    
    let req = client.post("https://nekobin.com/api/documents")
        .json(&json_data)
        .timeout(std::time::Duration::from_secs(5))
        .send()?;
    
    let response_json: serde_json::Value = req.json()?;
    let key = response_json["result"]["key"].as_str().unwrap();
    let url = format!("https://nekobin.com/{}", key);
    

    println!("{:?}", url);
    Ok(())
}
        

async fn handle_start_command(client: Client, message: grammers_client::types::Message) -> Result {
    let chat = message.chat();

    if is_chat_private(message).await {
        client
            .send_message(
                &chat,
                "Hello! I'm a bot written in Rust. I don't do much yet, but stay tuned!",
            )
            .await?;
    } else {
        client
            .send_message(
                &chat,
                "Hello! I;m Alive :)",
            )
            .await?;
    }

    Ok(())
}

async fn async_main() -> Result {
    let token = "6566300513:AAHA2b_SJ3C_Am4BrFsjNb6Vi8jHBLnqdJA";
    let api_id = 3138242;
    let api_hash = "9ff85074c961b349e6dad943e9b20f54";

    println!("RustyBot -> Connecting to Telegram...");
    let client = Client::connect(Config {
        session: Session::load_file_or_create("bot.session")?,
        api_id: api_id,
        api_hash: api_hash.to_string(),
        params: InitParams {
            // Fetch the updates we missed while we were offline
            catch_up: false,
            ..Default::default()
        },
    })
    .await?;
    println!("RustyBot -> Connected!");

    if !client.is_authorized().await? {
        println!("RustyBot -> Signing in...");
        client.bot_sign_in(token, api_id, api_hash).await?;
        client.session().save_to_file("bot.session")?;
    }

    let me = client.get_me().await?;
    if me.username() != Option::None {
        println!("RustyBot -> Signed in as @{:?}", me.username());
    } else {
        println!("RustyBot -> Signed in as a bot");
    }

    let chat = client
        .resolve_username("roseloverx")
        .await?
        .expect("chat not found");

    client.send_message(&chat, "Hello, world!").await?;

    // Handle Updates

    loop {
        let update = client.next_update().await;
        let upt = update?.unwrap();

        let handle = client.clone();
        task::spawn(async move {
            match handle_update(handle, upt).await {
                Ok(_) => {}
                Err(e) => eprintln!("Error handling updates!: {}", e),
            }
        });
    }
}

fn main() -> Result {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
