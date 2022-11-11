

#[castle_api::castle_macro(Type)]
pub(crate) struct SetupIntent {
    pub(crate) id: String,
    pub(crate) client_secret: String,
}

