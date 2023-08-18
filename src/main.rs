use grammers_client::{Client, Config, InitParams};
use grammers_session::Session;
use grammers_tl_types as tl;
use tokio::runtime;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

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

    let chat = client.resolve_username("roseloverx")
        .await?
        .expect("chat not found");

    client.send_message(&chat, "Hello, world!").await?;

    Ok(())
}

fn main() -> Result {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
