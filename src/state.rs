use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Hash, PartialEq, Eq)]
pub struct User {
    handle: String,
    addr: String,
    // public key
}

pub struct Conversation {
    //name: String, Implement when adding group messages
    partner: User, // Remove when adding group messages in favour of 'users'
    messages: Vec<String>,
    new_messages: VecDeque<String>,
    //users: map of all users in conversation. Implement when adding group messages.
}

impl Conversation {

    pub fn new(user: User) -> Conversation {
        Conversation {
            partner: user,
            messages: Vec::new(),
            new_messages: VecDeque::new(),
        }
    }
}

pub struct State {
    //conversations: HashMap<User, Conversation>,
    conversation: Option<Conversation>,
    known_users: HashSet<User>,
}

impl State {

    pub fn new() -> State {
        State {
            conversation: None,
            known_users: HashSet::new(),
        }
    }
}
