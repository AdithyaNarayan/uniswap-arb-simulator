use tokio::sync::broadcast::Sender;

use crate::liquidity_pool::{LiquidityPool, PoolMessage};

pub type ThreadSafePool = Box<dyn LiquidityPool + Send + Sync>;

pub struct Exchange {
    pub eth_dai_pool_1: ThreadSafePool,
    pub eth_dai_pool_2: ThreadSafePool,
}

impl Exchange {
    pub fn new(
        mut eth_dai_pool_1: ThreadSafePool,
        mut eth_dai_pool_2: ThreadSafePool,
        sender: Sender<PoolMessage>,
    ) -> Self {
        eth_dai_pool_1.set_sender(sender.clone());
        eth_dai_pool_2.set_sender(sender);

        Self {
            eth_dai_pool_1,
            eth_dai_pool_2,
        }
    }
}
