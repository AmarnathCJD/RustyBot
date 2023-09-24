use chatbot::chat_bot_handle;
use chatbot::is_query_chatbot;
use grammers_client::{Client, Config, InitParams, Update, InputMessage};
use grammers_session::Session;
use grammers_tl_types::types as tl;
use grammers_tl_types::enums as enums;
use tokio::{runtime, task};

mod dev;
    use dev::handle_exec;

mod chatbot;
mod misc;
    

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

async fn handle_update(client: Client, update: Update) -> Result {
    // println!("RustyBot -> Handling update: {:?}", update);
    match update {
        Update::NewMessage(message) if !message.outgoing() => {
            if message.text() == "/start" {
                handle_start_command(client, message).await?;
            } else if message.text() == "/ping" {
                handle_ping(client, message).await?;
            } else if message.text().to_string().starts_with("/paste") {
                misc::handle_paste(client, message).await?;
            } else if message.text().to_string().starts_with("/sh") || message.text().to_string().starts_with("/exec") {
                handle_exec(client, message).await?;
            } else if  message.text().to_string().starts_with("/info") {
                misc::get_info_handler(client, message).await?;
            } else {
                let msg = message.clone();
                if is_query_chatbot(msg).await {
                    chat_bot_handle(client, message).await?;
                }
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
    let token = "<BOT_TOKEN>";
    let api_id = 6;
    let api_hash = "<API_HASH>";

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

    let _ = ctrlc::set_handler(move || {
        std::process::exit(0);
    });

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
