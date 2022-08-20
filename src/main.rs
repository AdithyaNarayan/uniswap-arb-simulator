use std::time::Duration;

use constant_product_pool::ConstantProductPool;
use runner::runner;
use tokio::time::sleep;

mod arbitrage;
mod constant_product_pool;
mod exchange;
mod liquidity_pool;
mod runner;

#[tokio::main]
async fn main() {
    let eth_dai_pool_1 = ConstantProductPool::new(20f64, 50f64, 0.0);
    let eth_dai_pool_2 = ConstantProductPool::new(400f64, 1010f64, 0.0);

    runner(eth_dai_pool_1, eth_dai_pool_2, |exchange| async move {
        // Each block here simulates on transaction. A block is required
        // due to RAII for unlocking the RWLock
        {
            let mut exchange = exchange.write().await;

            let swap_amount = exchange.eth_dai_pool_1.amount_with_fees(1f64);

            let _ = exchange.eth_dai_pool_1.swap_a(swap_amount);
        }

        sleep(Duration::from_millis(200)).await;

        {
            let mut exchange = exchange.write().await;

            let swap_amount = exchange.eth_dai_pool_1.amount_with_fees(2f64);

            let _ = exchange.eth_dai_pool_1.swap_a(swap_amount);
        }

        drop(exchange);
    })
    .await;
}
