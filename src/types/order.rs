//! Order book implementation reworked for `#![no_std]` / allocator‑free setups.
//! Strings → `heapless::String`, Vecs → `heapless::Vec`.

use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt;

use heapless::{FnvIndexMap, String as HString, Vec as HVec};

use crate::types::interval::Interval;
use crate::types::rational::{err, ErrorString, Rational};

/// Tunables ----------------------------------------------------------------
/// Adjust at will; grow if you expect bigger order books or identifiers.
const MAX_ORDERS: usize = 32; // capacity of the per‑book Vec
const MAX_KEYS: usize = 16; // capacity of the ID → Order map
const STR_CAP: usize = 32; // capacity for IDs, asset symbols, …

/// Convenience alias for fixed‑capacity heapless strings.
pub type SmallStr = HString<STR_CAP>;

/// Public identifier for a market.
pub type MarketId = SmallStr;

#[derive(Clone, Debug)]
pub enum OrderType {
    LEND,
    BORROW,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    OPEN,
    PARTIALLY_FILLED,
    FILLED,
    EXPIRED,
}

#[derive(Clone, Debug)]
pub struct Order {
    pub order_id: SmallStr,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub asset: SmallStr,
    pub collateral: u128,
    pub amount: u128,
    pub remaining_amount: u128,
    pub vtl_range: Interval<Rational>,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order {{ id: {}, type: {:?}, status: {:?}, asset: {}, amount: {}, remaining: {}, vtl: [{}-{}] }}",
            self.order_id,
            self.order_type,
            self.status,
            self.asset,
            self.amount,
            self.remaining_amount,
            self.vtl_range.min,
            self.vtl_range.max,
        )
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub orders_by_vtl: HVec<Order, MAX_ORDERS>,
    pub orders_by_id: FnvIndexMap<SmallStr, Order, MAX_KEYS>,
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Render sorted by VTL
        write!(f, "by_vtl     [")?;
        let mut first = true;
        for o in &self.orders_by_vtl {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{}({}-{})", o.order_id, o.vtl_range.min, o.vtl_range.max)?;
        }
        write!(f, "]\n")?;

        // Render by ID
        write!(f, "by_id      [")?;
        let mut first = true;
        for key in self.orders_by_id.keys() {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{}", key)?;
        }
        write!(f, "]")
    }
}

#[derive(Debug)]
pub struct MarketOrderBooks {
    pub lender_book: OrderBook,
    pub borrower_book: OrderBook,
}

impl Order {
    pub fn new(
        order_id: &str,
        order_type: OrderType,
        asset: &str,
        collateral: u128,
        amount: u128,
        vtl_range: Interval<Rational>,
    ) -> Result<Self, ErrorString> {
        let order_id = SmallStr::try_from(order_id).map_err(|_| err("order_id too long"))?;

        let asset = SmallStr::try_from(asset).map_err(|_| err("asset too long"))?;

        Ok(Self {
            order_id,
            order_type,
            status: OrderStatus::OPEN,
            asset,
            collateral,
            amount,
            remaining_amount: amount,
            vtl_range,
        })
    }
}

impl OrderBook {
    pub const fn new() -> Self {
        Self {
            orders_by_vtl: HVec::new(),
            orders_by_id: FnvIndexMap::new(),
        }
    }

    pub fn iter_orders_by_id(&self) -> impl Iterator<Item = &Order> + '_ {
        self.orders_by_id.values()
    }

    pub fn iter_orders_by_vtl(&self) -> impl Iterator<Item = &Order> + '_ {
        self.orders_by_vtl.iter()
    }

    pub fn get_order_by_id(&self, id: &SmallStr) -> Option<&Order> {
        self.orders_by_id.get(id)
    }

    fn insert_sorted(
        vec: &mut HVec<Order, MAX_ORDERS>,
        order: Order,
        mut compare: impl FnMut(&Order, &Order) -> Ordering,
    ) {
        let idx = vec
            .binary_search_by(|probe| compare(probe, &order))
            .unwrap_or_else(|e| e);
        vec.insert(idx, order).ok(); // ignore capacity error (caller ensured room)
    }

    pub fn add_order(&mut self, order: Order) {
        if self.orders_by_vtl.len() == MAX_KEYS || self.orders_by_id.len() == MAX_KEYS {
            panic!("order book full");
        }
        let order_id = order.order_id.clone();
        let order_for_map = order.clone();
        Self::insert_sorted(&mut self.orders_by_vtl, order, |o1, o2| {
            let vtl_cmp = o1.vtl_range.cmp(&o2.vtl_range);
            if vtl_cmp != Ordering::Equal {
                vtl_cmp
            } else {
                o1.order_id.cmp(&o2.order_id)
            }
        });
        self.orders_by_id.insert(order_id, order_for_map).ok();
    }

    pub fn remove_order(&mut self, id: &SmallStr) -> Option<Order> {
        self.orders_by_id.remove(id)
    }
}
