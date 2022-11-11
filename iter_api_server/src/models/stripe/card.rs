

#[castle_api::castle_macro(Type)]
pub(crate) struct Card {
    pub(crate) id: String,
    pub(crate) last4: String,
    pub(crate) brand: String,
    pub(crate) nickname: String,
    pub(crate) card_holder: String,
}

