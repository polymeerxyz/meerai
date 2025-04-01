use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait, cast};

use crate::tools::bluesky::BskyToolset;

pub struct Bluesky {
    tool: BskyToolset,
}

/// This is the types of message [PingPong] supports
#[derive(Debug, Clone)]
pub enum Message {
    Ping,
    Pong,
}

impl Message {
    fn next(&self) -> Self {
        match self {
            Self::Ping => Self::Pong,
            Self::Pong => Self::Ping,
        }
    }

    fn print(&self) {
        match self {
            Self::Ping => print!("ping.."),
            Self::Pong => print!("pong.."),
        }
    }
}

#[async_trait]
impl Actor for Bluesky {
    type Msg = Message;
    type Arguments = ();
    type State = u8;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        // startup the event processing
        cast!(myself, Message::Ping)?;
        // create the initial state
        Ok(0u8)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if *state < 10u8 {
            message.print();
            cast!(myself, message.next())?;
            *state += 1;
        } else {
            println!();
            myself.stop(None);
            // don't send another message, rather stop the agent after 10 iterations
        }
        Ok(())
    }
}
