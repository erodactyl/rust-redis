use std::{collections::HashMap, hash::Hash, sync::mpsc, thread};

enum Message<K, V>
where
    K: Eq + Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    Get(K, mpsc::Sender<Option<V>>),
    Set(K, V),
    Remove(K),
}

#[derive(Clone)]
pub struct ThreadSafeMap<K, V>
where
    K: Eq + Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    tx: mpsc::Sender<Message<K, V>>,
}

impl<K, V> ThreadSafeMap<K, V>
where
    K: Eq + Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Message<K, V>>();

        thread::spawn(move || {
            let map: HashMap<K, V> = HashMap::new();

            Self::handle_messages(map, rx);
        });

        ThreadSafeMap { tx }
    }

    pub fn set(&self, key: K, value: V) {
        self.tx.send(Message::Set(key, value)).unwrap();
    }

    pub fn get(&self, key: K) -> Option<V> {
        let (tx, rx) = mpsc::channel::<Option<V>>();

        self.tx.send(Message::Get(key, tx)).unwrap();

        rx.recv().unwrap()
    }

    pub fn remove(&self, key: K) {
        self.tx.send(Message::Remove(key)).unwrap()
    }

    fn handle_messages(mut map: HashMap<K, V>, rx: mpsc::Receiver<Message<K, V>>) {
        rx.iter().for_each(|message| match message {
            Message::Get(key, tx) => {
                tx.send(map.get(&key).cloned()).unwrap();
            }
            Message::Set(key, value) => {
                map.insert(key, value);
            }
            Message::Remove(key) => {
                map.remove(&key);
            }
        });
    }
}
