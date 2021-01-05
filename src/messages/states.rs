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
