use std::collections::{HashMap, HashSet};

#[derive(Hash, PartialEq, Eq)]
pub struct User {
    handle: String,
    addr: String,
    // public key
}

pub struct Conversation {
    //name: String, Implement when adding group messages
    //partner: User, // Remove when adding group messages in favour of 'users'
    messages: Vec<String>,
    //users: map of all users in conversation. Implement when adding group messages.
}

impl Conversation {

    pub fn new() -> Conversation {
        Conversation {
            messages: Vec::new(),
        }
    }
}

pub struct State {
    //conversations: HashMap<User, Conversation>,
    conversation: Conversation,
    known_users: HashSet<User>,
}

impl State {

    pub fn new() -> State {
        State {
            conversation: Conversation::new(),
            known_users: HashSet::new(),
        }
    }
}
