use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{account::Accounts, connection::Sender, transaction::Transaction};

pub struct Generator;

impl Generator {
    pub async fn init<T, F>(
        flag: Flag,
        transaction_generator: T,
        accounts: Accounts,
        sender: Sender,
    ) where
        T: Fn(Accounts) -> F,
        F: Future<Output = Transaction> + Send + 'static,
    {
        while !flag.stopped() {
            let transaction = transaction_generator(accounts.clone()).await;
            sender.send(transaction).await.unwrap();
        }
    }
}

pub struct Flag(Arc<AtomicBool>);

impl Clone for Flag {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

impl Flag {
    pub fn stopped(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.0.store(true, Ordering::SeqCst)
    }
}
