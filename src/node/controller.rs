use crate::node::state::BoxedState;

/// The controller handles the logic with handling and emitting messages, as well as time based
/// actions, peer management, etc.
struct Controller {
    state: BoxedState,
}

impl Controller {}
