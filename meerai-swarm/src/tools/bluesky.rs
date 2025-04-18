use atrium_api::{app::bsky::feed::post::RecordData, types::string::Datetime};
use bsky_sdk::agent;
use meerai_core::{JsonSchema, ToolError, ToolOutput, async_trait};
use meerai_macros::Toolset;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct PostTweetArgs {
    pub text: String,
}

#[derive(Toolset)]
#[toolset(
    tool(
        name = "Post Tweet",
        description = "Post a tweet to Bluesky (bsky.app) given the content.",
        params = PostTweetArgs,
    )
)]
pub struct BskyToolset {
    bsky_client: agent::BskyAgent,
}

impl BskyToolset {
    pub fn new(bsky_client: agent::BskyAgent) -> Self {
        Self { bsky_client }
    }
}

#[async_trait]
impl BskyToolsetInvoke for BskyToolset {
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
