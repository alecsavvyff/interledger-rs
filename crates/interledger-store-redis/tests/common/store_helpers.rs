use super::fixtures::*;
use super::redis_helpers::*;
use env_logger;
use futures::Future;
use interledger_api::NodeStore;
use interledger_store_redis::{Account, RedisStore, RedisStoreBuilder};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

pub fn test_store() -> impl Future<Item = (RedisStore, TestContext, Vec<Account>), Error = ()> {
    let context = TestContext::new();
    RedisStoreBuilder::new(context.get_client_connection_info(), [0; 32])
        .connect()
        .and_then(|store| {
            let store_clone = store.clone();
            let mut accs = Vec::new();
            store
                .clone()
                .insert_account(ACCOUNT_DETAILS_0.clone())
                .and_then(move |acc| {
                    accs.push(acc.clone());
                    store_clone
                        .insert_account(ACCOUNT_DETAILS_1.clone())
                        .and_then(move |acc| {
                            accs.push(acc.clone());
                            Ok((store, context, accs))
                        })
                })
        })
}

pub fn block_on<F>(f: F) -> Result<F::Item, F::Error>
where
    F: Future + Send + 'static,
    F::Item: Send,
    F::Error: Send,
{
    // Only run one test at a time
    let _ = env_logger::try_init();
    let lock = TEST_MUTEX.lock();
    let mut runtime = Runtime::new().unwrap();
    let result = runtime.block_on(f);
    drop(lock);
    result
}
