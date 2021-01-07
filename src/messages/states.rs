/// Describes at which point of the Whoami protocol
/// a node is.
/// 
/// A pair of `WhoamiState` is needed to keep track
/// of the local and the remote node.
#[derive(Debug, PartialEq)]
pub enum WhoamiSate {
    Unkn,   // We didn't engage the whoami protocol
    Sent,   // The whoami message has been sent
    Ack,   // The whoami_ack message has been received / sent
}

pub const WHOAMI_MSG: &str = "whoami";
pub const WHOAMIACK_MSG: &str = "whoamiack";

pub const VERSION: u32 = 1;
pub const SERVICES: [&str; 1] = ["node"];


#[derive(Debug, PartialEq)]
pub enum CurrentAction {
    WaitingHeader,  // Default mode : the node is waiting for a new message.
    WaitingWhoami(u64),  // Whoami size
}

#[derive(PartialEq)]
pub enum PingType {
    Ping,
    Pong,
}

#[derive(Debug, PartialEq)]
pub enum PingState {
    Sent,
    Ack,
}

pub const PING_MSG: &str = "2plus2is4";
pub const PONG_MSG: &str = "minus1thats3";

/// A ping message is supposed to be
/// sent and received each `PING_CALLBACK` secs.
pub const PING_CALLBACK: u8 = 42;
pub const LAST_SEEN_THRESHOLD: u32 = 300;

/// Magic number, used in the header
pub const MAGIC: u32 = 422021;
