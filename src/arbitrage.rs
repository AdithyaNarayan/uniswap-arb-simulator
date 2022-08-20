use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{constant_product_pool::ConstantProductPool, exchange::Exchange};

pub async fn run_arbitrage(exchange: Arc<RwLock<Exchange>>) {
    let pool_1;
    {
        pool_1 = exchange
            .read()
            .await
            .eth_dai_pool_1
            .as_any()
            .downcast_ref::<ConstantProductPool>()
            .expect("Only implemented for ConstantProductPool")
            .read_only_copy();
    }
    let pool_2;
    {
        pool_2 = exchange
            .read()
            .await
            .eth_dai_pool_2
            .as_any()
            .downcast_ref::<ConstantProductPool>()
            .expect("Only implemented for ConstantProductPool")
            .read_only_copy();
    }
    let a = pool_1.amount_a;
    let b = pool_1.amount_b;

    let c = pool_2.amount_a;
    let d = pool_2.amount_b;

    let a_plus_delta_a =
        (a * b * (b + d) + (a * b * c * d * (b + d).powi(2)).sqrt()) / (b + d).powi(2);

    let input_amount_a = a_plus_delta_a - a;

    if input_amount_a.abs() < 1e-9 {
        println!("No arbitrage opportunities found!");
        return;
    }

    let output_amount_a;
    {
        let mut txn = exchange.write().await;
        let tmp_amount_b = txn.eth_dai_pool_1.swap_a(input_amount_a).unwrap();
        output_amount_a = txn.eth_dai_pool_2.swap_b(tmp_amount_b).unwrap();

        println!(
            "Required ETH: {}, Send into Pool {}",
            input_amount_a.abs(),
            if input_amount_a > 0.0 { 1 } else { 2 }
        );
        println!("Profit: {}", output_amount_a - input_amount_a);
    }
}
