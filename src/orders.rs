use crate::{errors::OrderErrors, DataParser};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone)]
pub struct Orders {
    pub order: BTreeMap<usize, Order>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub index: usize,
    pub client_name: String,
    pub operation: OrderType,
    pub asset: String,
    pub order_price: u32,
    pub value: u32,
}

type Result<T> = std::result::Result<T, OrderErrors>;

// types of orders
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderType {
    Buy,
    Sell,
    IsNotOrderType,
}

impl Orders {
    //get mut from order btreemap
    pub fn get_mut<T>(&mut self, order_index: T) -> Option<&mut Order>
    where
        T: Into<usize>,
    {
        let order_index = &order_index.into();
        self.order.get_mut(order_index)
    }

    //get from order btreemap
    pub fn get<T>(&self, order_index: T) -> Option<Order>
    where
        T: Into<usize>,
    {
        let order_index = &order_index.into();
        Some(self.order.get(order_index)?.clone())
    }

    //A function that updates the values ​​of unfulfilled orders 
    pub fn update_orders(&mut self, updated_orders: Vec<Order>) {
        for updated_order in updated_orders {
            let mut order = self.get_mut(updated_order.index).unwrap();
            order.value = updated_order.value;
        }
    }
}


impl DataParser for Orders {
    type Item = Order;
    type Err = OrderErrors;

    fn new() -> Orders {
        Orders {
            order: BTreeMap::new(),
        }
    }

    fn insert(&mut self, index: usize, mut data: Self::Item) {
        data.index = index;
        self.order.insert(index, data);
    }

    fn parse(line: &str) -> Result<Self::Item> {
        Order::from_str(line)
    }

    fn remove(&mut self, orders: Vec<Self::Item>) {
        for order in orders {
            self.order.remove(&order.index);
        }
    }
}

// FromStr impl for Order struct.
impl FromStr for Order {
    type Err = OrderErrors;
    // Converting a String to an Order Structure
    fn from_str(s: &str) -> Result<Self> {
        use OrderErrors::*;
        let vals: Vec<&str> = s.split_whitespace().collect();

        if vals.len() < 5 {
            return Err(ParseInsufficentInputError);
        }

        // The index has been added for easier interaction with the map.
        let index = 0;

        let client_name = vals[0].parse::<String>().map_err(|_| ParseClientIdError)?;

        let operation = match vals[1].parse::<char>().map_err(|_| ParseOperationError)? {
            'b' => OrderType::Buy,
            's' => OrderType::Sell,
            _ => OrderType::IsNotOrderType,
        };

        if operation == OrderType::IsNotOrderType {
            return Err(NoSuchOperationSymbolError);
        }

        let asset = vals[2].parse::<String>().map_err(|_| ParseSymbolError)?;
        let order_price = vals[3].parse::<u32>().map_err(|_| ParseItemPriceError)?;
        let value = vals[4].parse::<u32>().map_err(|_| ParseItemVolumeError)?;

        Ok(Order {
            index,
            client_name,
            operation,
            asset,
            order_price,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let base_string = "C5    b    C    15    4";
        let struct_exemplar = Order::from_str(base_string).unwrap();

        assert_eq!(struct_exemplar.client_name, "C5");
        assert_eq!(struct_exemplar.operation, OrderType::Buy);
        assert_eq!(struct_exemplar.asset, "C");
        assert_eq!(struct_exemplar.order_price, 15);
        assert_eq!(struct_exemplar.value, 4);
    }

    #[test]
    fn test_parse_insufficent_error_from_str() {
        let base_string = "C5    b    C    15    ";
        let actual_error = Order::from_str(base_string).unwrap_err();
        let expected_error = OrderErrors::ParseInsufficentInputError;
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn test_parse_operation_error() {
        let base_string = "C5    24    C    15    4";
        let actual_error = Order::from_str(base_string).unwrap_err();
        let expected_error = OrderErrors::ParseOperationError;
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn test_no_such_operation_error() {
        let base_string = "C5    c    C    15    4";
        let actual_error = Order::from_str(base_string).unwrap_err();
        let expected_error = OrderErrors::NoSuchOperationSymbolError;
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn test_item_price_error() {
        let base_string = "C5    b    C    a    4";
        let actual_error = Order::from_str(base_string).unwrap_err();
        let expected_error = OrderErrors::ParseItemPriceError;
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn test_item_volume_error() {
        let base_string = "C5    b    C    15    a";
        let actual_error = Order::from_str(base_string).unwrap_err();
        let expected_error = OrderErrors::ParseItemVolumeError;
        assert_eq!(actual_error, expected_error);
    }
}