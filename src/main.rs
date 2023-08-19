use grammers_client::{Client, Config, InitParams, Update};
use grammers_session::Session;
use grammers_tl_types as tl;
use tokio::{runtime, task};
user std::time

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

async fn handle_update(client: Client, update: Update) -> Result {
    // println!("RustyBot -> Handling update: {:?}", update);
    match update {
        Update::NewMessage(message) if !message.outgoing() => {
            if message.text() == "/start" {
                handle_start_command(client, message).await?;
            } else if message.text() == "/ping" {
                handle_ping(client, message).await?;
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
    let start_time = std::time::Instant::now(); // Record the start time

    let msg = client.send_message(&chat, "Pong").await?; // Send "Pong" message

    let end_time = std::time::Instant::now(); // Record the end time
    let elapsed_time = end_time - start_time; // Calculate elapsed time

    client.send_message(&chat, format!("Ping response time: {:?}", elapsed_time)).await?; // Send response time message

    Ok(()) // Return Ok to indicate success
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
    let token = "6419307556:AAGDyUmmYntQkyoY8AFljUTWjT3ARK8Y6lk";
    let api_id = 3138242;
    let api_hash = "9ff85074c961b349e6dad943e9b20f54";

    println!("RustyBot -> Connecting to Telegram...");
    let client = Client::connect(Config {
        session: Session::load_file_or_create("bot.session")?,
        api_id: api_id,
        api_hash: api_hash.to_string(),
        params: InitParams {
            // Fetch the updates we missed while we were offline
            catch_up: true,
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
