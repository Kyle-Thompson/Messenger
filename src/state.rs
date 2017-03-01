use std::collections::{HashMap, HashSet};

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

pub struct State {
    conversations: HashMap<User, Conversation>,
    known_users: HashSet<User>,
}
