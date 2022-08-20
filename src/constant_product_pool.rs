use anyhow::{anyhow, Result};
use std::any::Any;
use tokio::sync::broadcast::Sender;

use crate::liquidity_pool::{Downcastable, LiquidityPool, PoolMessage};

#[derive(Debug, Clone, Default)]
pub struct ConstantProductPool {
    pub amount_a: f64,
    pub amount_b: f64,

    pub fee: f64,

    pub sender: Option<Sender<PoolMessage>>,
}

impl ConstantProductPool {
    pub fn k(&self) -> f64 {
        self.amount_a * self.amount_b
    }

    pub fn new(amount_a: f64, amount_b: f64, fee: f64) -> Self {
        Self {
            amount_a,
            amount_b,
            fee,

            ..Default::default()
        }
    }

    pub fn read_only_copy(&self) -> Self {
        Self {
            amount_a: self.amount_a,
            amount_b: self.amount_b,

            fee: self.fee,

            sender: None,
        }
    }
}

impl Downcastable for ConstantProductPool {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl LiquidityPool for ConstantProductPool {
    fn add(&mut self, amount_a_to_add: f64) {
        self.amount_b += (self.amount_b * amount_a_to_add) / self.amount_a;

        self.amount_a += amount_a_to_add;

        self.sender.as_ref().map(|s| s.send(PoolMessage::LP));
    }

    fn remove(&mut self, amount_a_to_remove: f64) -> Result<()> {
        if amount_a_to_remove > self.amount_a
        // check redundant
        // || (self.amount_b * amount_a_to_remove) / self.amount_a > self.amount_b
        {
            return Err(anyhow!("amount to remove greater than liquidity in pool"));
        }

        self.amount_b -= (self.amount_b * amount_a_to_remove) / self.amount_a;
        self.amount_a -= amount_a_to_remove;

        self.sender.as_ref().map(|s| s.send(PoolMessage::LP));
        Ok(())
    }

    // Theoretically works with negative values
    fn swap_a(&mut self, input_amount_a_with_fees: f64) -> Result<f64> {
        let input_amount_a = input_amount_a_with_fees / (1f64 + (self.fee / 100f64));

        let output_amount_b = self.amount_b - self.k() / (self.amount_a + input_amount_a);

        if output_amount_b > self.amount_b {
            return Err(anyhow!("amount to swap greater than liquidity in pool"));
        }

        self.amount_a += input_amount_a_with_fees;
        self.amount_b -= output_amount_b;

        self.sender.as_ref().map(|s| s.send(PoolMessage::Swap));
        Ok(output_amount_b)
    }

    fn swap_b(&mut self, input_amount_b_with_fees: f64) -> Result<f64> {
        // subtracting fees to get real amount
        let input_amount_b = input_amount_b_with_fees / (1f64 + (self.fee / 100f64));

        let output_amount_a = self.amount_a - self.k() / (self.amount_b + input_amount_b);

        if output_amount_a > self.amount_a {
            return Err(anyhow!("amount to swap greater than liquidity in pool"));
        }

        self.amount_b += input_amount_b_with_fees;
        self.amount_a -= output_amount_a;

        self.sender.as_ref().map(|s| s.send(PoolMessage::Swap));
        Ok(output_amount_a)
    }

    fn amount_with_fees(&self, amount: f64) -> f64 {
        amount * (1f64 + (self.fee / 100f64))
    }

    fn set_sender(&mut self, sender: Sender<PoolMessage>) {
        self.sender = Some(sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_equal_lp_add() {
        let mut pool = ConstantProductPool::new(5f64, 10f64, 0.3);

        pool.add(20f64);

        assert_eq!(pool.amount_a, 25f64);
        assert_eq!(pool.amount_b, 50f64);
    }

    #[test]
    fn test_value_equal_lp_remove() -> Result<()> {
        let mut pool = ConstantProductPool::new(10f64, 20f64, 0.3);

        pool.remove(5f64)?;

        assert_eq!(pool.amount_a, 5f64);
        assert_eq!(pool.amount_b, 10f64);

        Ok(())
    }

    #[test]
    fn test_remove_lp_exceeded() {
        let mut pool = ConstantProductPool::new(10f64, 20f64, 0.3);

        assert!(pool.remove(20f64).is_err());
    }

    #[test]
    fn test_swap_simple() -> Result<()> {
        let mut pool = ConstantProductPool::new(10f64, 50f64, 0.3);
        let old_k = pool.k();

        let output = pool.swap_a(pool.amount_with_fees(1.0))?;

        assert_eq!(format!("{:.3}", output), "4.545");
        assert!(old_k < pool.k());
        assert_eq!(pool.amount_a, 11.003);

        Ok(())
    }

    #[test]
    fn test_assert_k_same_if_zero_fees() -> Result<()> {
        let mut pool = ConstantProductPool::new(10f64, 50f64, 0.0);
        let old_k = pool.k();

        let output = pool.swap_a(pool.amount_with_fees(1.0))?;

        assert_eq!(format!("{:.3}", output), "4.545");
        assert_eq!(old_k, pool.k());
        assert_eq!(pool.amount_a, 11f64);

        Ok(())
    }
}
