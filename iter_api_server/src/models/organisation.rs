use std::str::FromStr;

use castle_api::types::State;
use mongodb::{bson::{doc, oid::ObjectId, bson},};
use serde::{Deserialize, Serialize};
use stripe::{PaymentMethodId, PriceId, SubscriptionItem, Object};
use super::{User, utils::{model::Model, stripe::get_stripe_client}, OrganisationMember, Profile, stripe::product::get_current_subscription, page::Page};

#[derive(Debug, Serialize, Deserialize)]
pub struct Organisation {
    _id: ObjectId,
    name: String,
    //date_created: Date,
    //created_by: ObjectID,
    // display_picture: Option<String>,
    legal_name: String,
    billing_address: String,
}

impl Model for Organisation {
    fn collection_name() ->  &'static str {
        "organisations"
    }
}

impl Organisation {
    /// Create a new organisation in the database.
    ///
    /// ## Algorithm
    /// 1. Create the [Organisation] with required information
    /// 2. Create the first [Profile] and attach it to the organisation
    /// 3. Create the first [OrganisationMember] and attach it to the organisation
    /// 4. Return the [Organisation] [ObjectId]
    // directive authenticated
    pub async fn create_organisation(state: &State, name: &str) -> Result<ObjectId, anyhow::Error> {
        let user = state.borrow::<User>();

        let organisation_id = Organisation::create(state, bson!({
            "name": name,
            "created_by": user._id,
            "date_created": mongodb::bson::DateTime::now(),
            "display_picture": None::<ObjectId>,
            "legal_name": "",
            "billing_address": "",
        })).await?;

        let profile_id = Profile::create(state, bson!({
            "name": format!("{} {}", user.first_name, user.last_name),
            "display_picture": None::<String>,
            "organisation_id": organisation_id,
            "roles": vec!["admin"],
            "username": String::new()
        })).await?;

        OrganisationMember::create(state, bson!({
            "organisation_id": organisation_id,
            "date_joined": mongodb::bson::DateTime::now(),
            "date_invited": mongodb::bson::DateTime::now(),
            "email": user.email.clone(),
            "user_id": user._id,
            "profiles": vec![profile_id]
        })).await?;

        Ok(organisation_id)
    }

}

#[castle_api::castle_macro(Type)]
impl Organisation {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }
    pub fn name(&self, _state: &State) -> &str {
        &self.name
    }

    pub async fn update_name(&self, state: &State, name: &str) -> Result<(), anyhow::Error> {
        Organisation::update(state, &self._id, doc!{
            "$set": {
                "name": name
            }
        }).await?;
        Ok(())
    }

    pub fn legal_name(&self, _state: &State) -> &str {
        &self.legal_name
    }

    pub async fn update_legal_name(&self, state: &State, legal_name: &str) -> Result<(), anyhow::Error> {
        Organisation::update(state, &self._id, doc!{
            "$set": {
                "legal_name": legal_name
            }
        }).await?;
        Ok(())
    }

    pub fn billing_address(&self, _state: &State) -> &str {
        &self.billing_address
    }

    pub async fn update_billing_address(&self, state: &State, billing_address: &str) -> Result<(), anyhow::Error> {
        Organisation::update(state, &self._id, doc!{
            "$set": {
                "billing_address": billing_address
            }
        }).await?;
        Ok(())
    }

    pub async fn members(&self, state: &State) -> Result<Vec<OrganisationMember>, anyhow::Error> {
        OrganisationMember::find_many(
            state,
            doc!{

                "organisation_id": self._id,
            },
            100
        ).await
    }

    pub async fn member_count(&self, state: &State) -> Result<u64, anyhow::Error> {
        OrganisationMember::count(state, doc!{
            "organisation_id": self._id,
        }).await
    }

    pub async fn profiles(
        &self,
        state: &State
    ) -> Result<Vec<Profile>, anyhow::Error> {
        Ok(Profile::find_many(
            state,
            doc!{
                "organisation_id": self._id
            },
            100
        ).await?)
    }
    // pub fn profile_picture(&self, _state: &State) -> Option<String> {
    //     self.display_picture.clone()
    // }


    // adding member is done in OrganisationMember

    // gets or creates a stripe customer id if it doesn't exist
    pub async fn stripe_customer_id(
        &self,
        state: &State
    ) -> Result<String, anyhow::Error> {
        match Self::get_field_optional::<String>(&self._id, "stripe_customer_id", state).await? {
            Some(id) => Ok(id),
            None => {
                let client = get_stripe_client(state);

                let customer = stripe::Customer::create(&client, stripe::CreateCustomer {
                    name: Some(&self.name),
                    // email: Some(&state.borrow::<User>().email),
                    metadata: Some([
                        (String::from("organisation_id"), self._id.to_hex())
                    ].into_iter().collect()),
                    ..Default::default()
                }).await?;

                Organisation::update(state, &self._id, doc!{
                    "$set": {
                        "stripe_customer_id": customer.id.as_str()
                    }
                }).await?;

                Ok(String::from(customer.id.as_str()))
            }
        }
    }

    async fn stripe_create_setup_intent(&self, state: &State) -> Result<super::stripe::setup_intent::SetupIntent, anyhow::Error> {
        let client = get_stripe_client(state);
        let intent = stripe::SetupIntent::create(&client, stripe::CreateSetupIntent {
            customer: Some(stripe::CustomerId::from_str(
                &self.stripe_customer_id(state).await?
            )?),
            ..Default::default()
        }).await?;

        Ok(super::stripe::setup_intent::SetupIntent {
            id: intent.id.to_string(),
            client_secret: intent.client_secret.ok_or(anyhow::anyhow!("No client secret"))?
        })
    }

    async fn card_payment_methods(&self, state: &State) -> Result<Vec<super::stripe::card::Card>, anyhow::Error> {
        let client = get_stripe_client(state);
        let payment_methods = stripe::PaymentMethod::list(&client, &stripe::ListPaymentMethods {
            type_: stripe::PaymentMethodTypeFilter::Card,
            customer: Some(stripe::CustomerId::from_str(
                &self.stripe_customer_id(state).await?
            )?),
            limit: Some(100),
            ending_before: None,
            expand: &[],
            starting_after: None,
        }).await?;

        payment_methods.data.into_iter().map(|pm| {

            let card = pm.card.ok_or(anyhow::anyhow!("No card"))?;

            Ok(super::stripe::card::Card {
                id: pm.id.to_string(),
                brand: card.brand.to_string(),
                last4: card.last4.to_string(),
                nickname: pm.metadata.get("nickname").map(|s| s.to_string()).unwrap_or(String::from("")),
                card_holder: pm.billing_details.name.clone().unwrap_or(String::from(""))
            })
        }).collect()
    }

    async fn set_stripe_default_payment_method(&self, state: &State, payment_method_id: String) -> Result<(), anyhow::Error> {
        let client = get_stripe_client(state);
        stripe::Customer::update(&client, &stripe::CustomerId::from_str(
            &self.stripe_customer_id(state).await?
        )?, stripe::UpdateCustomer {
            invoice_settings: Some(stripe::CustomerInvoiceSettings {
                default_payment_method: Some(payment_method_id),
                ..Default::default()
            }),
            ..Default::default()
        }).await?;

        Ok(())
    }

    async fn stripe_default_payment_method(&self, state: &State) -> Result<Option<String>, anyhow::Error> {
        let client = get_stripe_client(state);
        let customer = stripe::Customer::retrieve(&client, &stripe::CustomerId::from_str(
            &self.stripe_customer_id(state).await?
        )?, &[]).await?;

        match customer.invoice_settings
            .map(|settings| settings.default_payment_method
                .map(|expandable |
                    expandable.id().to_string())).flatten() {
            Some(id) => Ok(Some(id)),
            None => match self.card_payment_methods(state).await?.get(0) {
                Some(card) => {
                    self.set_stripe_default_payment_method(state, card.id.clone()).await?;
                    Ok(Some(card.id.clone()))
                },
                None => Ok(None)
            }
        }
    }

    async fn current_price_id(&self, state: &State) -> Result<String, anyhow::Error>{
        let client = get_stripe_client(state);
        return match get_current_subscription(&client, &self.stripe_customer_id(state).await?).await? {
            Some(stripe::Subscription{cancel_at_period_end, canceled_at, cancel_at, items, ..}) => { 
                if cancel_at_period_end || canceled_at.is_some() || cancel_at.is_some() {
                    return Ok("".to_string())
                }
                match items.data.get(0) {
                Some(subscription_item) => match &subscription_item.price {
                    Some(price) => Ok(price.id().as_str().to_string()),
                    None => Ok("".to_string())
                }
                None => Ok("".to_string())
            }},
            None => Ok("".to_string())
        }
    }

    async fn update_subscription(&self, state: &State, price_id: String) -> Result<(), anyhow::Error> {
        let client = get_stripe_client(state);
        
        // get the default payment method
        match self.stripe_default_payment_method(state).await? {
            None => return Err(anyhow::anyhow!("You need to add a payment method")),
            _ => ()
        }
        
        let existing_subscription = get_current_subscription(&client, &self.stripe_customer_id(state).await?).await?;
        let license_quantity = self.member_count(state).await?;
        match existing_subscription {
            Some(subscription) => match subscription.items.data.get(0) {
                    Some(SubscriptionItem { id, .. }) => {
                        let update_subscription = stripe::UpdateSubscription { cancel_at_period_end: Some(false), cancel_at: None, ..Default::default() };
                        stripe::Subscription::update(&client, &subscription.id(), update_subscription).await?;
                        stripe::SubscriptionItem::update(&client, id, stripe::UpdateSubscriptionItem {
                            price: Some(PriceId::from_str(&price_id)?),
                            quantity: Some(license_quantity),
                            ..Default::default()
                        }).await?;
                        return Ok(())
                    },
                    // cancel the existing subscription, because interval is not the same, or malformed
                    _ => {  }
                },
                None => {}
            };


        // create a subscription because there was none, or it was cancelled
        stripe::Subscription::create(&client, stripe::CreateSubscription {
            items: Some(vec![
                stripe::CreateSubscriptionItems {
                    price: Some(price_id),
                    quantity: Some(license_quantity),
                    ..Default::default()
                }
            ]),
            ..stripe::CreateSubscription::new(stripe::CustomerId::from_str(
                &self.stripe_customer_id(state).await?
            )?)
        }).await?;

        Ok(())
    }
    async fn cancel_subscription(&self, state: &State) -> Result<(), anyhow::Error> {
        let client = get_stripe_client(state);
        let subscription_id = match get_current_subscription(&client, &self.stripe_customer_id(state).await?).await?{
            Some(subscription) => subscription.id(),
            _ => return Err(anyhow::anyhow!("You need to have a subscription to cancel it"))
        };
        let update_subscription = stripe::UpdateSubscription { cancel_at_period_end: Some(true), ..Default::default() };
        stripe::Subscription::update(&client, &subscription_id, update_subscription).await?;
        Ok(())
    }

    // async fn create_subscription(&self, price_id: String, state: State) -> Result<(), anyhow::Error> {
    //     todo!();
    //     // let users_len = unimplemented!();
    //     // let item = json!({
    //     //     "price": price_id,
    //     //     "quantity":
    //     // });
    // }

    async fn remove_payment_method(&self, state: &State, payment_method_id: String) -> Result<(), anyhow::Error> {
        let client = get_stripe_client(state);
        stripe::PaymentMethod::detach(&client, &PaymentMethodId::from_str(&payment_method_id)?).await?;
        Ok(())
    }

    pub async fn root_pages(&self, state: &State) -> Result<Vec<Page>, anyhow::Error> {
        Page::find_many(
            state,
            doc!{
                "organisation": self._id,
                "parent_page": None::<ObjectId>
            },
            100
        ).await
    }
}


