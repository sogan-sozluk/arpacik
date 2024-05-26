use sea_orm_migration::sea_query::{Expr, SimpleExpr};

pub fn current_timestamp_utc() -> SimpleExpr {
    Expr::cust("timezone('UTC', now())")
}
