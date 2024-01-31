#![cfg_attr(not(feature = "std"), no_std, no_main)]



#[ink::contract]
mod lobstah {
    use ink::prelude::vec; // Importing vec from ink prelude
    use ink::storage::Mapping;
    use scale::Output;
    
    pub type OrderId = u64;

    pub type Price = u16;

    pub type Size = u64;

    pub type Side = u8;

    #[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Order {
        pub symbol: String,
        pub trader: String,
        pub side: Side,
        pub price: Price,
        pub size: Size,
    }
    #[derive(Debug, PartialEq, Eq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct OrderIn {
        order: Order,
        id: OrderId,
    }
    
    #[ink(storage)]
    #[derive(Default)]
    pub struct Lobstah {
        bids: Vec<OrderIn>,
        asks: Vec<OrderIn>,
        id: OrderId,
    }
    
    impl Lobstah {
        pub fn is_ask(s: Side) -> bool { return s == 1; }

        // Helpers for cross
        pub fn hit_ask(bid: Price, ask: Price) -> bool {
            return bid >= ask;
        }

        pub fn hit_bid(ask: Price, bid: Price) -> bool {
            return ask <= bid;
        }

        // Helpers for queue
        pub fn priority_ask(ask_new: Price, ask_old: Price) -> bool {
            return ask_new < ask_old;
        }

        pub fn priority_bid(bid_new: Price, bid_old: Price) -> bool {
            return bid_new > bid_old;
        }
       
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self {
                bids: Vec::<OrderIn>::new(),
                asks: Vec::<OrderIn>::new(),
                id: 1,
            }
        }
        
        #[ink(message, payable)]
        pub fn limit_order(&mut self, ord: Order) -> OrderId {
            // Cross off as many shares as possible.
            let mut order = ord.clone();
            let isask = Self::is_ask(order.side);
            let book = if isask { &mut self.bids } else { &mut self.asks };
            let cross_test = if isask { Self::hit_bid } else { Self::hit_ask };
            
            for matched_order in book.iter_mut() {
                if order.size == 0 {
                    break;
                }
                if !cross_test(order.price, matched_order.order.price) {
                    break;
                }
                if order.size >= matched_order.order.size {
                    order.size -= matched_order.order.size;
                    // Removed via retain operation in cross
                    matched_order.order.size = 0;
                }
                // New order completely filled.
                else {
                    matched_order.order.size -= order.size;
                    order.size = 0;
                }   
            }
            book.retain(|x| x.order.size > 0);
            if order.size > 0 {
                // Queue order if all shares not crossed off.
                let insertion_index = match book.iter().enumerate().find(|(_index, ele)| cross_test(order.price, ele.order.price)) {
                    Some((a, _)) => a,
                    _ => book.len(),
                };
                                    
                let new_order = OrderIn { order: order, id: self.id };
                book.insert(insertion_index, new_order);
            }
            let return_id = self.id;
            self.id += 1;
            return_id
        }
    
        #[ink(message)]
        pub fn cancel(&mut self, id: OrderId) {
            self.asks.retain(|x| x.id != id);
            self.bids.retain(|x| x.id != id);
        }
    }

}
