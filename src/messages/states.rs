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

#[derive(Debug, PartialEq)]
pub enum CurrentAction {
    Nothing,
}

#[derive(PartialEq)]
pub enum PingType {
    Ping,
    Pong,
}

pub const PING_MSG: &str = "2plus2is4";
pub const PONG_MSG: &str = "minus1thats3";

/// A ping message is supposed to be
/// sent and received each `PING_CALLBACK` secs.
pub const PING_CALLBACK: u8 = 42;
