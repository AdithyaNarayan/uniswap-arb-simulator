use anyhow::Result;
use std::{any::Any, fmt::Debug};
use tokio::sync::broadcast::Sender;

#[derive(Debug, Clone)]
pub enum PoolMessage {
    Init,
    Swap,
    LP,
}

pub trait Downcastable: Any {
    fn as_any(&self) -> &dyn Any;
}

pub trait LiquidityPool: Debug + Downcastable {
    // No auth checks for adding/removing LP
    // Automatically adds equal value of B
    fn add(&mut self, amount_a_to_add: f64);
    fn remove(&mut self, amount_a_to_remove: f64) -> Result<()>;

    // Amount includes fees
    fn swap_a(&mut self, input_amount_a: f64) -> Result<f64>;
    fn swap_b(&mut self, input_amount_b: f64) -> Result<f64>;

    fn amount_with_fees(&self, amount: f64) -> f64;

    fn set_sender(&mut self, sender: Sender<PoolMessage>);
}
