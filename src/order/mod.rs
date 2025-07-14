use crate::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Buy,
    Sell,
}

impl TryFrom<&str> for Side {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "buy" => Ok(Side::Buy),
            "sell" => Ok(Side::Sell),
            _ => eyre::bail!("Invalid order side: '{}'", value),
        }
    }
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderType {
    Market,
    Limit, // 暂不详细处理限价单逻辑
}

#[derive(Debug, Clone)]
pub struct Order {
    // pub exchange: String,
    // pub symbol: Symbol,
    // pub order_id: String,
    // pub price: f64,
    // pub quantity: f64,
    // pub side: OrderSide,
    // pub status: OrderStatus,
}
