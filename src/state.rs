use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::clone::Clone;

use net_lib::TextMessage;

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct User {
    handle: String,
    addr: String,
    // public key
}

pub struct Conversation {
    //name: String, Implement when adding group messages
    partner: User, // Remove when adding group messages in favour of 'users'
    messages: Vec<String>,
    new_messages: VecDeque<TextMessage>,
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

type Conversations = HashMap<String, Conversation>;

pub struct State {
    //conversations: HashMap<User, Conversation>,
    conversations: Arc<(Mutex<Conversations>, Condvar)>,
    current_conversation: Option<Conversation>,
    known_users: HashSet<User>,
    unseen_messages: u32,
}

impl State {

    pub fn new() -> State {
        State {
            conversations: Arc::new((Mutex::new(Conversations::new()), Condvar::new())),
            current_conversation: None,
            known_users: HashSet::new(),
            unseen_messages: 0,
        }
    }

    pub fn add_new_message(&self, msg: TextMessage) {
        let &(ref mutex, ref cvar) = &*self.conversations;
        let ref mut conv: Conversations = *mutex.lock().unwrap();
        let ref mut conv = conv.entry(msg.clone().conv_id).or_insert(Conversation::new(msg.clone().sender));

        conv.new_messages.push_back(msg);
        cvar.notify_one();
    }
}
