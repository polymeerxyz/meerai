use agent_twitter_client::scraper::Scraper;
use anyhow::{Error, Result};
use meerai_core::{JsonSchema, ToolError, ToolOutput, async_trait};
use meerai_macros::{Schema, Toolset};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Schema)]
pub struct ReadTweetArgs {
    pub url: String,
}

#[derive(Toolset)]
#[toolset(
    tool(
        name = "Read Tweet",
        description = "Read a tweet from X (x.com / twitter.com) given the URL.",
        params = ReadTweetArgs,
    )
)]
pub struct XToolset {
    scraper: Scraper,
}

impl XToolset {
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

#[async_trait]
impl XToolsetInvoke for XToolset {
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
