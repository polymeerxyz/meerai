use agent_twitter_client::scraper::Scraper;
use anyhow::{Error, Result};
use meerai_macros::Toolset;

#[derive(Toolset)]
#[toolset(
    name = "X Tool",
    description = "Read a tweet from X (x.com / twitter.com) given the URL.",
    tool(
        name = "read_tweet",
        description = "Read a tweet from X (x.com / twitter.com) given the URL.",
        param(
            name = "url",
            r#type = "string",
            required = true,
            description = "The URL of the tweet to read"
        )
    )
)]
pub struct XTool {
    scraper: Scraper,
}

impl XTool {
    pub fn new(scraper: Scraper) -> Self {
        Self { scraper }
    }

    fn extract_tweet_id(url: &str) -> Result<String> {
        url.split('/')
            .next_back()
            .map(ToString::to_string)
            .ok_or(Error::msg("Invalid URL"))
    }
}

#[async_trait::async_trait]
impl Invoke for XTool {
    async fn read_tweet(&self, args: &ReadTweetArgs) -> Result<ToolOutput, ToolError> {
        let id = Self::extract_tweet_id(&args.url)?;
        self.scraper
            .get_tweet(&id)
            .await
            .map_err(|err| {
                ToolError::Unknown(anyhow::anyhow!("Failed to read tweet: {}", err.to_string()))
            })
            .map(|response| ToolOutput::Text(response.text.unwrap_or_default()))
    }
}
