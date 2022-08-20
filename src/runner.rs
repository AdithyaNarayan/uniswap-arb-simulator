use std::{future::Future, sync::Arc, time::Duration};

use tokio::{
    sync::{broadcast, Mutex, RwLock},
    time::{sleep, Instant},
};

use crate::{
    arbitrage::run_arbitrage, constant_product_pool::ConstantProductPool, exchange::Exchange,
    liquidity_pool::PoolMessage,
};

pub async fn runner<T: Future<Output = ()>>(
    pool_1: ConstantProductPool,
    pool_2: ConstantProductPool,
    code: fn(Arc<RwLock<Exchange>>) -> T,
) {
    let eth_dai_pool_1 = Box::new(pool_1);
    let eth_dai_pool_2 = Box::new(pool_2);
    let (sender, mut listener) = broadcast::channel(10000000);

    let thread_safe_exchange = Arc::new(RwLock::new(Exchange::new(
        eth_dai_pool_1,
        eth_dai_pool_2,
        sender.clone(),
    )));

    let total_duration = Arc::new(Mutex::new(Duration::ZERO));
    let num_of_iter = Arc::new(Mutex::new(0));

    let exchange_handle = thread_safe_exchange.clone();
    let duration_handle = total_duration.clone();
    let num_handle = num_of_iter.clone();

    tokio::spawn(async move {
        while let Ok(msg) = listener.recv().await {
            match msg {
                PoolMessage::Init | PoolMessage::Swap => {
                    let start_time = Instant::now();

                    run_arbitrage(exchange_handle.clone()).await;

                    *duration_handle.clone().lock().await += start_time.elapsed();
                    *num_handle.clone().lock().await += 1;
                }
                PoolMessage::LP => (),
            }
        }
    });

    let _ = sender.send(PoolMessage::Init);

    code(thread_safe_exchange.clone()).await;

    sleep(Duration::from_secs(1)).await;

    println!(
        "Average duration of each arbitrage calculation: {:?}",
        *total_duration.lock().await / *num_of_iter.lock().await
    );
}
