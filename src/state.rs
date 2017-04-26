#![allow(dead_code)]

use std::collections::{HashMap};
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::clone::Clone;

extern crate rand;

use messages::TextMessage;
use net_lib::Net;
use crypto_lib::Key;
use mpmc_queue::MpmcQueue;

pub type AddrPair = (String, Key);
pub type Route = Vec<AddrPair>;

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct User {
    pub handle: String,
    pub addr: String,
    pub public_key: Key,
}

impl User {

    pub fn new(handle: String, addr: String, key: Key) -> User {
        User {
            handle: handle,
            addr: addr,
            public_key: key
        }
    }

    pub fn from_addr_pair(handle: String, pair: &AddrPair) -> User {
        User {
            handle: handle.to_string(),
            addr: pair.0.clone(),
            public_key: pair.1.clone()
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Conversation {
    //name: String, Implement when adding group messages
    partner: User, // Remove when adding group messages in favour of 'users'
    messages: Vec<TextMessage>,
    new_message_count: usize,
    id: u64,
    priv_id: usize,
    //users: map of all users in conversation. Implement when adding group messages.
}

impl Conversation {

    pub fn new(user: User) -> Conversation {
        Conversation {
            partner: user,
            messages: Vec::new(),
            new_message_count: 0,
            id: rand::random::<u64>(),
            priv_id: Conversation::next_id(),
        }
    }

    pub fn from_id(user: User, id: u64) -> Conversation {
        Conversation {
            partner: user,
            messages: Vec::new(),
            new_message_count: 0,
            id: id,
            priv_id: Conversation::next_id(),
        }
    }

    fn next_id() -> usize {
        static N: AtomicUsize = ATOMIC_USIZE_INIT;
        N.fetch_add(1, Ordering::SeqCst)
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_priv_id(&self) -> usize {
        self.priv_id
    }

    pub fn new_message_count(&self) -> usize {
        self.new_message_count
    }

    pub fn inc_new_msg_count(&mut self) {
        self.new_message_count += 1;
    }

    pub fn set_new_message_count(&mut self, count: usize) {
        self.new_message_count = count;
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
    users: Arc<Mutex<HashMap<String, Route>>>,
}

impl State {

    pub fn new() -> State {
        State {
            conversations: Arc::new((Mutex::new(Conversations::new()), Condvar::new())),
            current_conversation: Arc::new(Mutex::new(None)),
            unseen_message_count: Arc::new(Mutex::new(0)),
            channel: Arc::new(MpmcQueue::new()),
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_new_message(&self, msg: TextMessage) {
        self.current_conversation.lock().unwrap().map_or_else(
            || *self.unseen_message_count.lock().unwrap() += 1,
            |curr|
                if curr == msg.conv_id { self.channel.push(msg.clone()); } 
                else { *self.unseen_message_count.lock().unwrap() += 1; }
        );

        let &(ref mutex, ref cvar) = &*self.conversations;
        mutex.lock().and_then(|mut convs| {
            let conv = convs.entry(msg.conv_id)
                .or_insert(Conversation::from_id(msg.sender.clone(), msg.conv_id));
            conv.messages.push(msg.clone());
            conv.inc_new_msg_count();
            Ok(())
        }).unwrap();

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

    pub fn get_message_history(&self) -> Option<Vec<TextMessage>> {
        self.current_conversation.lock().unwrap()
            .and_then(|curr| {
                self.conversations.0.lock().unwrap()
                    .get(&curr)
                    .and_then(|c| Some(c.messages.clone()))
            })
        
    }

    pub fn set_current_conversation(&self, conv: Option<u64>) -> Result<(), &'static str> {
        *self.current_conversation.lock().unwrap() = conv;

        conv.map_or_else(
            || Ok(()), 
            |new_conv| self.conversations.0.lock().unwrap()
                .get_mut(&new_conv)
                .and_then(|conv| {
                    conv.set_new_message_count(0);
                    Some(())
                })
                .ok_or("Conversation does not exist."))
    }

    pub fn list_conversations(&self) -> Vec<String> {
        self.conversations.0.lock().unwrap().values()
            .map(|c| format!("{} [{}]: {}", 
                             c.get_priv_id(), 
                             c.new_message_count(), 
                             c.get_partner().handle))
            .collect()
    }

    pub fn conv_name_to_id(&self, name: &str) -> Option<u64> {
        self.conversations.0.lock().unwrap().values()
            .find(|&c| c.get_partner().handle.trim() == name.trim())
            .and_then(|c| Some(c.get_id()))
    }

    pub fn get_route(&self, user: &str, net: &Net) -> Result<Route, String> {
        match self.users.lock().unwrap().entry(user.to_string()) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => net.get_route(&user).map(|ui| v.insert(ui).clone())
        }
    }
}

