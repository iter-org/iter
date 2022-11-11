use std::str::FromStr;

use castle_api::types::State;
use stripe::{ListProducts, ListPrices, RecurringInterval};

use crate::models::utils::stripe::get_stripe_client;





#[castle_api::castle_macro(Type)]
pub(crate)struct Product {
    pub(crate) product_id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) yearly_price: PlanPrice,
    pub(crate) monthly_price: PlanPrice,
}

#[castle_api::castle_macro(Type)]
pub(crate) struct PlanPrice {
    pub(crate) price_id: String,
    pub(crate) product_id: String,
    pub(crate) amount_in_cents: i64,
}

impl Product {

    pub async fn get_products(state: &State) -> Result<Vec<Product>, anyhow::Error> {
        let client = get_stripe_client(state);

        let products = stripe::Product::list(&client, &ListProducts {
            ..Default::default()
        }).await?.data;

        let mut all_products = vec![];

        for product in products {
            let prices = stripe::Price::list(&client, &ListPrices {
                product: Some(stripe::IdOrCreate::Id(&product.id)),
                ..Default::default()
            }).await?.data;

            let yearly_price = prices.iter().find(|price| price.recurring.as_ref().map(|r| r.interval == RecurringInterval::Year).unwrap_or(false))
                .ok_or(anyhow::anyhow!("No yearly price found for product {}", product.id))?;
            let monthly_price = prices.iter().find(|price| price.recurring.as_ref().map(|r| r.interval == RecurringInterval::Month).unwrap_or(false))
                .ok_or(anyhow::anyhow!("No monthly price found for product {}", product.id))?;

            let product = Product {
                product_id: product.id.to_string(),
                name: product.name.ok_or(anyhow::anyhow!("No name found"))?,
                description: product.description.unwrap_or("".to_string()),
                yearly_price: PlanPrice {
                    price_id: yearly_price.id.to_string(),
                    product_id: product.id.to_string(),
                    amount_in_cents: yearly_price.unit_amount.unwrap() as i64,
                },
                monthly_price: PlanPrice {
                    price_id: monthly_price.id.to_string(),
                    product_id: product.id.to_string(),
                    amount_in_cents: monthly_price.unit_amount.unwrap() as i64,
                },
            };

            all_products.push(product);

        }
        all_products.sort_by(|a, b| a.monthly_price.amount_in_cents.cmp(&b.monthly_price.amount_in_cents));

        Ok(all_products)
    }
}

pub async fn get_current_subscription(client: &stripe::Client, stripe_customer_id: &str) -> Result<Option<stripe::Subscription>, anyhow::Error> {
    let existing_subscription = stripe::Subscription::list(&client, &stripe::ListSubscriptions {
        customer: Some(stripe::CustomerId::from_str(stripe_customer_id)?),
        limit: Some(1),
        ..Default::default()
    }).await?.data.pop();
    return Ok(existing_subscription)
}