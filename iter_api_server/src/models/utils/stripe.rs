use std::sync::Arc;

use castle_api::types::State;
use secrets::BackendSecrets;

pub(crate) fn get_stripe_client(state: &State) -> stripe::Client {
    let secrets = state.borrow::<Arc<BackendSecrets>>();

    stripe::Client::new(secrets.stripe_secrets.secret_key.clone())
}