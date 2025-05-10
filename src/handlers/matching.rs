use core::cmp::Ordering;

use crate::types::order::{Order, OrderBook, OrderStatus};

/// Find the first open borrow order whose VTL range overlaps with the lend_order
pub fn match_lend<'a>(borrow_orderbook: &'a OrderBook, lend_order: &'a Order) -> Option<&'a Order> {
    let lend_min = &lend_order.vtl_range.min;
    let lend_max = &lend_order.vtl_range.max;

    // binary search for first entry with min > lend_min
    let mut lo = 0;
    let mut hi = borrow_orderbook.orders_by_vtl.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let probe = &borrow_orderbook.orders_by_vtl[mid];
        let cmp = if &probe.vtl_range.min <= lend_min {
            Ordering::Less
        } else {
            Ordering::Greater
        };
        if cmp == Ordering::Less {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let idx = lo;

    // scan for overlap
    for o in &borrow_orderbook.orders_by_vtl[idx..] {
        if o.status != OrderStatus::OPEN {
            continue;
        }
        let lower = o.vtl_range.min.clone().max(lend_min.clone());
        let upper = o.vtl_range.max.clone().min(lend_max.clone());
        if lower <= upper {
            return Some(o);
        }
    }
    None
}

/// Find the best lend order whose VTL range overlaps with the borrow_order
pub fn match_borrow<'a>(
    lend_orderbook: &'a OrderBook,
    borrow_order: &'a Order,
) -> Option<&'a Order> {
    let borrow_min = &borrow_order.vtl_range.min;
    let borrow_max = &borrow_order.vtl_range.max;

    // find insertion point = first lend.min > borrow_min
    let idx = lend_orderbook
        .orders_by_vtl
        .binary_search_by(|probe| {
            if &probe.vtl_range.min <= borrow_min {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
        .unwrap_or_else(|i| i);

    // if idx == 0, no lend.min <= borrow_min
    if idx == 0 {
        return None;
    }

    let o = &lend_orderbook.orders_by_vtl[idx - 1];
    if o.status != OrderStatus::OPEN {
        return None;
    }

    let lower = o.vtl_range.min.clone().max(borrow_min.clone());
    let upper = o.vtl_range.max.clone().min(borrow_max.clone());
    if lower > upper {
        return None;
    }

    Some(o)
}
