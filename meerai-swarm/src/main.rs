use std::sync::Arc;

use agent_twitter_client::scraper::Scraper;
use bsky_sdk::BskyAgent;
use dotenv::dotenv;
use meerai_core::{OpenRouterBuilder, agent::multi_turn_agent::MultiTurnAgentBuilder};
use meerai_swarm::tools;

fn init_logging() {
    let dir = tracing_subscriber::filter::Directive::from(tracing::Level::DEBUG);

    use std::io::IsTerminal;
    use std::io::stderr;
    use tracing_glog::Glog;
    use tracing_glog::GlogFields;
    use tracing_subscriber::Registry;
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;

    let fmt = tracing_subscriber::fmt::Layer::default()
        .with_ansi(stderr().is_terminal())
        .with_writer(std::io::stderr)
        .event_format(Glog::default().with_timer(tracing_glog::LocalTime::default()))
        .fmt_fields(GlogFields::default().compact());

    let filter = vec![dir]
        .into_iter()
        .fold(EnvFilter::from_default_env(), |filter, directive| {
            filter.add_directive(directive)
        });

    let subscriber = Registry::default().with(filter).with(fmt);
    tracing::subscriber::set_global_default(subscriber).expect("to set global subscriber");
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logging();

    let cookie = std::env::var("X_COOKIE_STRING").expect("X_COOKIE_STRING not set");
    let mut scraper = Scraper::new().await.unwrap();
    scraper.set_from_cookie_string(&cookie).await.unwrap();

    let agent = BskyAgent::builder().build().await.unwrap();
    let session = agent.login("", "").await.unwrap();

    println!("Logged in to BSky: {session:?}");

    let llm_client = OpenRouterBuilder::default()
        .build()
        .expect("Failed to create OpenRouter client");

    let mut agent = MultiTurnAgentBuilder::default()
        .chat_completion(Arc::new(llm_client))
        .tools(vec![
            Arc::new(tools::x::XTool::new(scraper)),
            Arc::new(tools::bluesky::BskyTool::new(agent.clone())),
            Arc::new(meerai_common::Stop {}),
        ])
        .build()
        .unwrap();

    let response = agent
        .prompt("Read a tweet from X https://x.com/AltcoinDailyio/status/1906810513562239021, rephrase the content and then post the new version to Bluesky")
        .await
        .expect("Failed to prompt LLM");

    println!("LLM: {response:?}");
}
