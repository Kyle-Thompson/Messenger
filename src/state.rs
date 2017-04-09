use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::clone::Clone;

extern crate rand;

use net_lib::TextMessage;
use mpmc_queue::MpmcQueue;

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct User {
    pub handle: String,
    pub route: Vec<String>,
    // public key
}

#[derive(Clone, PartialEq)]
pub struct Conversation {
    //name: String, Implement when adding group messages
    partner: User, // Remove when adding group messages in favour of 'users'
    messages: Vec<TextMessage>,
    new_messages: VecDeque<TextMessage>,
    id: u64,
    //users: map of all users in conversation. Implement when adding group messages.
}

impl Conversation {

    pub fn new(user: User) -> Conversation {
        Conversation {
            partner: user,
            messages: Vec::new(),
            new_messages: VecDeque::new(),
            id: rand::random::<u64>(),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_partner(&self) -> &User {
        &self.partner
    }
}

type Conversations = HashMap<u64, Conversation>;

pub struct NewMessagesIter<'a> {
    state: &'a State,
}

impl<'a> Iterator for NewMessagesIter<'a> {
    type Item = TextMessage;

    fn next(&mut self) -> Option<TextMessage> {
        Some(self.state.channel.pop())
    }
}

pub struct State {
    conversations: Arc<(Mutex<Conversations>, Condvar)>,
    current_conversation: Arc<Mutex<Option<u64>>>,
    unseen_message_count: Arc<Mutex<u32>>,
    channel: Arc<MpmcQueue<TextMessage>>,
}

impl State {

    pub fn new() -> State {
        State {
            conversations: Arc::new((Mutex::new(Conversations::new()), Condvar::new())),
            current_conversation: Arc::new(Mutex::new(None)),
            unseen_message_count: Arc::new(Mutex::new(0)),
            channel: Arc::new(MpmcQueue::new()),
        }
    }

    pub fn add_new_message(&self, msg: TextMessage) {
        let &(ref mutex, ref cvar) = &*self.conversations;
        let convs: &mut Conversations = &mut *mutex.lock().unwrap();
        let conv: &mut Conversation = convs.entry(msg.clone().conv_id)
            .or_insert(Conversation::new(msg.clone().sender));

        // TODO: Fix this garbage.
        if let Some(ref s) = *self.current_conversation.lock().unwrap() {
            if *s == msg.clone().conv_id {
                self.channel.push(msg.clone());
                conv.messages.push(msg);
            } else {
                conv.new_messages.push_back(msg);
                *self.unseen_message_count.lock().unwrap() += 1;
            }
        } else {
            conv.new_messages.push_back(msg);
            *self.unseen_message_count.lock().unwrap() += 1;
        }

        cvar.notify_one();
    }

    pub fn get_new_messages(&self) -> NewMessagesIter {
        NewMessagesIter {
            state: &self,
        }
    }
    
    pub fn get_current_conversation(&self) -> Option<Conversation> {
        self.current_conversation.lock().unwrap().and_then(|u| {
            self.conversations.0.lock().unwrap().get(&u).map(|x| x.clone())
        })
    }

    pub fn add_conversation(&self, conv: Conversation) {
        self.conversations.0.lock().unwrap().insert(conv.get_id(), conv);
    }

    pub fn set_current_conversation(&self, new_conv: Option<u64>) {
        *self.current_conversation.lock().unwrap() = new_conv;
    }
}

