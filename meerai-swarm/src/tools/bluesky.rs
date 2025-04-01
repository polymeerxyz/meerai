use atrium_api::{app::bsky::feed::post::RecordData, types::string::Datetime};
use bsky_sdk::agent;
use meerai_macros::Toolset;

#[derive(Toolset)]
#[toolset(
    name = "Bsky Tool",
    description = "Post a tweet to Bluesky (bsky.app) given the content.",
    tool(
        name = "post_tweet",
        description = "Post a tweet to Bluesky (bsky.app) given the content.",
        param(
            name = "text",
            r#type = "string",
            required = true,
            description = "The content of the tweet to post"
        )
    )
)]
pub struct BskyTool {
    bsky_client: agent::BskyAgent,
}

impl BskyTool {
    pub fn new(bsky_client: agent::BskyAgent) -> Self {
        Self { bsky_client }
    }
}

#[async_trait::async_trait]
impl Invoke for BskyTool {
    async fn post_tweet(&self, args: &PostTweetArgs) -> Result<ToolOutput, ToolError> {
        self.bsky_client
            .create_record(RecordData {
                created_at: Datetime::now(),
                embed: None,
                entities: None,
                facets: None,
                labels: None,
                langs: None,
                reply: None,
                tags: None,
                text: args.text.clone(),
            })
            .await
            .map_err(|err| {
                ToolError::Unknown(anyhow::anyhow!("Failed to post tweet: {}", err.to_string()))
            })
            .map(|response| {
                ToolOutput::Text(format!("Tweet posted successfully: {}", response.uri))
            })
    }
}
