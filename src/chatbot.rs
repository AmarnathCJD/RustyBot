use grammers_client::Client;
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;

type CommandResult = std::result::Result<(), Box<dyn std::error::Error>>;

struct GPT {
    authkey: String,
    query_limit: u32,
    parent_id: String,
    convos: HashMap<i64, String>,
}

// define gpt struct methods

impl GPT {
    fn new(authkey: String, parent_id: String) -> GPT {
        GPT {
            authkey: authkey,
            query_limit: 100,
            parent_id: parent_id,
            convos: HashMap::new(),
        }
    }

    async fn query(
        &mut self,
        prompt: String,
        chat_id: i64,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let chat_conv_id = &self.convos.get(&chat_id);
        let body: serde_json::Value;

        if chat_conv_id.is_none() {
            body = json!({
                "text": prompt,
                "action": "new",
                "parentId": &self.parent_id,
                "workspaceId": &self.parent_id,
                "messagePersona": "default",
                "model": "gpt-3.5-turbo",
                "internetMode": "never",
                "hidden": false
            });
        } else {
            body = json!({
                "text": prompt,
                "action": "continue",
                "parentId": &self.parent_id,
                "workspaceId": &self.parent_id,
                "messagePersona": "default",
                "model": "gpt-3.5-turbo",
                "internetMode": "never",
                "hidden": false,
                "id": chat_conv_id
            });
        }

        let url = format!("https://streaming-worker.forefront.workers.dev/chat");
        let client = reqwest::Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&("Bearer ".to_owned() + &self.authkey))?,
        );

        let res = client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?
            .text()
            .await?;

        let (text, conv_chat_id) = parse_response(chat_conv_id.is_none(), res).await;
        if chat_conv_id.is_none() {
            self.convos.insert(chat_id, conv_chat_id.to_string());
        }

        Ok((text.to_string(), chat_id.to_string()))
    }
}

async fn parse_response(new: bool, response: String) -> (String, String) {
    let mut chat_id = "";
    if new {
        let re = Regex::new(r#""chatId":"(.*)""#).unwrap();
        chat_id = re.captures(&response).unwrap().get(1).unwrap().as_str();
    }

    let mut split = response.split("event: end");
    let mut text = split.nth(1).unwrap().to_string();
    text = text.split("data: ").last().unwrap().to_string();
    let text_w = text.split("\"").nth(1);
    if text_w.is_some() {
        text = text_w.unwrap().to_string();
    } else {
        text = "I don't know what to say".to_string();
    }

    (text, chat_id.to_string())
}

pub async fn is_query_chatbot(message: grammers_client::types::Message) -> bool {
    if let grammers_client::types::Chat::User(_) = message.chat() {
        if message.reply_to_message_id().unwrap_or(0) != 0 {
            return true;
        }
    } else {
        if message.text().to_string().to_lowercase().contains("rusty") {
            return true;
        } else if message.reply_to_message_id().unwrap_or(0) != 0 {
            let reply = message.get_reply().await.unwrap();
            if reply
                .unwrap()
                .sender()
                .unwrap()
                .username()
                .unwrap()
                .to_lowercase()
                == "rustydbot"
            {
                return true;
            }
        }
    }
    return false;
}

static mut GPT_CLIENT: Option<GPT> = None;
static mut GPT_INDEX: i32 = 0;
static mut GPT_CRED: Option<Vec<(String, String)>> = None;

pub async fn chat_bot_handle(
    _client: Client,
    message: grammers_client::types::Message,
) -> CommandResult {
    let chat_id = message.chat().id();
    let mut query = message.text().to_string();
    if message.text().contains("/gpt") {
        query = query.replace("/gpt", "");
    }

    unsafe {
        if GPT_CRED.is_none() {
            GPT_CRED =  Some(vec![
            ("eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yTzZ3UTFYd3dxVFdXUWUyQ1VYZHZ2bnNaY2UiLCJ0eXAiOiJKV1QifQ.eyJhenAiOiJodHRwczovL2NoYXQuZm9yZWZyb250LmFpIiwiZXhwIjoxNjkzMjE5MDI3LCJpYXQiOjE2OTMyMTg5NjcsImlzcyI6Imh0dHBzOi8vY2xlcmsuZm9yZWZyb250LmFpIiwibmJmIjoxNjkzMjE4OTU3LCJzaWQiOiJzZXNzXzJVYnIzaER4UW1mcmlBcDllRXRSOTlKU25GWCIsInN1YiI6InVzZXJfMlVicjNndHA0ekNOSVlyU2pTeTNVZ1lpa1Z3In0.KwLMJheZUZpzdlf4ASbLvxeA1w4kamBU3D0IblzUg5SOXKtIAk39P9vWRB5oBEYqeqoye3w0S4YvDX-8fHRUmJGii2QcZgbbN4i1-d3r272xwHvOQAfE6i_rWsWdq56dJCfw8yOHG403C_rIe94K7_J9rPfv6dZ050PFg69qr-btF7Ss8XH0BWuu5s_VC073BEpAhCvSR6uaqWUM0hALAgUI933pEKtsQclumYjEy20PEdE7hm1oELczxm1PnPOERqfEilg58Q2e8TRjWvyZdaz2fdHxB-KdwPYauZFUYpVYSeHFvI3CJ6i4Y5AG-OwdmPfSPCF1tTKgIFgDI2g9xQ".to_string(), "ddb3d027-a96d-491b-8352-64ba6c301808".to_string()),
            ("eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yTzZ3UTFYd3dxVFdXUWUyQ1VYZHZ2bnNaY2UiLCJ0eXAiOiJKV1QifQ.eyJhenAiOiJodHRwczovL2NoYXQuZm9yZWZyb250LmFpIiwiZXhwIjoxNjkzMjMxODAzLCJpYXQiOjE2OTMyMzE3NDMsImlzcyI6Imh0dHBzOi8vY2xlcmsuZm9yZWZyb250LmFpIiwibmJmIjoxNjkzMjMxNzMzLCJzaWQiOiJzZXNzXzJVY0dzQmQ1R1JWRU9mOWZVT3BMd2R1N2FhZiIsInN1YiI6InVzZXJfMlVjR3NCNE5pakQyNDhNb1UyRUlZSEpoM1RHIn0.eSL7GAm_urS9q2tkXDz6ZT5v5yPU2DhuCe7DqzFJbw_XNlNB1pG0MxzHJiZzOM895LgI_C43h_dm_9E6cCIykYwtcHGn8YG1Zxxb7qIsUrZHT0pZcDdu_0tijsgsoOkGXZ2hgAvwPS1LTG7kEYILYyrenc8croZqTZsbW1fUljjDf_XnzOznEykFVVWvqy3xH55dC713N_t5Srz0b82C-Qmte5y_NcGZJHJynojaADNLGtvtrthnHKotvTPCH7lS2AICKZO0dZCVYU3CwL6pCyMVnMjbyL7j423TERHhgRcdbS0XD-Dm43SrDERl6G3VyIg4b6CWtW8J-R9dOfGifw".to_string(), "80969c53-03cb-427c-8161-1ab1e497e293".to_string())
            ])
        }

        if GPT_CLIENT.is_none() {
            let mut _client = GPT::new(
                GPT_CRED.as_ref().unwrap()[GPT_INDEX as usize].0.to_string(),
                GPT_CRED.as_ref().unwrap()[GPT_INDEX as usize].1.to_string(),
            );
            GPT_CLIENT = Some(_client);
        }
    }
    let _client = unsafe { GPT_CLIENT.as_mut().unwrap() };

    let result = _client.query(query, chat_id).await.unwrap();
    let text = result.0.replace("Jenna", message.sender().unwrap().name()).replace("\n", "\\n");

    if let grammers_client::types::Chat::User(_) = message.chat() {
        message.respond(text).await?;
    } else {
        message.reply(text).await?;
    }
    Ok(())
}
