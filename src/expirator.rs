use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use tokio::{sync::{mpsc::Receiver, Mutex, RwLock}, time::sleep};

use log::info;

#[derive(Clone)]
pub struct Expirator {
    rx: Arc<Mutex<Receiver<(String, Duration)>>>,
    cache: Arc<DashMap<String, String>>
}

impl Expirator {
    pub(crate) fn new(rx: Arc<Mutex<Receiver<(String, Duration)>>>, cache: Arc<DashMap<String, String>>) -> Expirator {
        Expirator { rx, cache }
    }

    pub async fn listen(self: &Self) {
        loop {
            let mut receiver = self.rx.lock().await;
            match receiver.recv().await {
                Some((key, expiry)) => {
                    let cache_clone = self.cache.clone();
                    tokio::spawn(async move {
                        let _ = sleep(expiry).await;
                        info!(target: "expirator", "deleting key {key:?}");
                        cache_clone.remove(key.as_str());
                    });
                    ()
                },
                None => todo!(),
            }
        }
    }
}
