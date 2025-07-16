use agent_twitter_client::scraper::Scraper;
use dotenv::dotenv;
use meerai_core::OpenRouter;
use meerai_swarm::{config::load_config, log::init_logging, tools, workers::bluesky::BlueskyActor};
use ractor::Actor;

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

    let llm_client = OpenRouter::default();

    // Clone config and create tools first to avoid partial moves
    let bluesky_config = config.bluesky.clone();
    let x_toolset = tools::XToolset::new(scraper);

    // Create BlueskyActor which internally manages its tools
    let bluesky_actor = BlueskyActor::new(bluesky_config, llm_client.clone())
        .await
        .expect("Failed to create BlueskyActor");

    let prompt = "Read a tweet from X https://x.com/AltcoinDailyio/status/1906810513562239021, rephrase the content and then post the new version to Bluesky";

    // Run with BlueskyActor
    let (actor, actor_handle) = Actor::spawn(Some("bluesky-actor".to_string()), bluesky_actor, ())
        .await
        .expect("Failed to spawn BlueskyActor");

    actor_handle.await.expect("Failed to send prompt");
}
