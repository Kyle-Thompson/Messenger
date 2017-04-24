#![allow(dead_code)]

use std::collections::{HashMap};
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use std::clone::Clone;
//use std::io::{self, Write};
extern crate rand;

use net_lib::TextMessage;
use net_lib::Net;
use crypto_lib::KeyArr;
use mpmc_queue::MpmcQueue;

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct UserInfo {
    pub route: Vec<(String, KeyArr)>,
    pub addr: String, // see if this can be replaced by a call to the last element in route.
    pub public_key: [u8; 32],
}

#[derive(Clone, RustcEncodable, RustcDecodable, Hash, PartialEq, Eq)]
pub struct User {
    pub handle: String,
    pub addr: String, // replace this will addr again soon.
    pub public_key: [u8; 32],
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
    users: Arc<Mutex<HashMap<String, UserInfo>>>,
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
        let &(ref mutex, ref cvar) = &*self.conversations;

        let convs: &mut Conversations = &mut *mutex.lock().unwrap();
        let conv: &mut Conversation = convs.entry(msg.clone().conv_id)
            .or_insert(Conversation::from_id(msg.clone().sender, msg.clone().conv_id));
        conv.messages.push(msg.clone());
        conv.inc_new_msg_count();

        // TODO: Fix this garbage.
        if let Some(ref s) = *self.current_conversation.lock().unwrap() {
            if *s == msg.clone().conv_id {
                self.channel.push(msg.clone());
            } else {
                *self.unseen_message_count.lock().unwrap() += 1;
            }
        } else {
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

    pub fn set_current_conversation(&self, new_conv: Option<u64>) -> Option<Vec<TextMessage>> {
        *self.current_conversation.lock().unwrap() = new_conv;
        if let Some(conv) = new_conv {
            let ref mut curr = *self.conversations.0.lock().unwrap();
            let mut curr2: &mut Conversation = match curr.get_mut(&conv) {
                Some(c) => c,
                None => {
                    panic!("failed in set_current_conversation");
                }
            };
            curr2.set_new_message_count(0);
            let mut c = curr2.messages.clone();
            c.reverse(); // TODO: this doesn't need to be reversed.
            Some(c)
        } else {
            None
        }
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
        for conv in self.conversations.0.lock().unwrap().values() {
            if conv.get_partner().handle.trim() == name.trim() {
                return Some(conv.get_id());
            }
        }
        None
    }

    pub fn get_user_info(&self, user: &str, net: &Net) -> Result<UserInfo, String> {
        match self.users.lock().unwrap().entry(user.to_string()) {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => {
                net.get_user_info(&user).map(|ui| v.insert(ui).clone())
                /*match net.get_user_info(&user) {
                    Ok(ui) => {
                        Ok(v.insert(ui).clone())
                    },
                    Err(e) => Err(e.to_string())
                }*/
            }
        }
    }
}

