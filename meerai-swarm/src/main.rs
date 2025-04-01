use agent_twitter_client::scraper::Scraper;
use bsky_sdk::BskyAgent;
use dotenv::dotenv;
use meerai_agents::agents::MultiTurnAgent;
use meerai_core::OpenRouterBuilder;
use meerai_swarm::{config::load_config, log::init_logging, tools};

const DEFAULT_SYSTEM_PROMPT: &str = "
You are a **Senior Assistant Specialist**.

Your role is to help the user successfully complete their request by utilizing all available tools at your disposal.

- Tasks may require multiple steps to complete.
- For each step, if a tool is needed, you should invoke it with the appropriate arguments.
- Once the tool responds, assess the result and determine the next appropriate step.
- Continue this process until the task is fully completed.
- When the task is complete, you must use the **special tool** called **stop** to signal the end of the task.
";

#[tokio::main]
async fn main() {
    init_logging();

    dotenv().ok();
    let config = load_config().expect("Failed to load config");

    let mut scraper = Scraper::new().await.unwrap();
    scraper
        .set_from_cookie_string(&config.x.cookie)
        .await
        .unwrap();

    let agent = BskyAgent::builder().build().await.unwrap();
    let session = agent
        .login(config.bluesky.identifier, config.bluesky.password)
        .await
        .unwrap();

    let llm_client = OpenRouterBuilder::default()
        .build()
        .expect("Failed to create OpenRouter client");

    let mut agent = MultiTurnAgent::new(
        llm_client,
        vec![
            Box::pin(tools::XToolset::new(scraper)),
            Box::pin(tools::BskyToolset::new(agent.clone())),
            Box::pin(meerai_common::toolsets::StopToolset {}),
            Box::pin(meerai_common::toolsets::StopWithReasonToolset {}),
        ],
        DEFAULT_SYSTEM_PROMPT.to_string(),
    );

    let prompt_result = agent
        .prompt("Read a tweet from X https://x.com/AltcoinDailyio/status/1906810513562239021, rephrase the content and then post the new version to Bluesky")
        .await;
    let response = prompt_result.expect("Failed to get response from LLM");

    println!("LLM: {response:?}");
}
