use deno_core::error::AnyError;
use deno_error::JsErrorClass;
use std::borrow::Cow;

#[derive(Debug)]
pub struct OpError(pub AnyError);

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for OpError {}

impl JsErrorClass for OpError {
    fn get_class(&self) -> Cow<'static, str> {
        "Error".into()
    }
    fn get_message(&self) -> Cow<'static, str> {
        self.0.to_string().into()
    }
    fn get_additional_properties(&self) -> Box<dyn Iterator<Item = (Cow<'static, str>, deno_error::PropertyValue)> + 'static> {
        Box::new(std::iter::empty())
    }
    fn get_ref(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }
}

impl From<AnyError> for OpError {
    fn from(err: AnyError) -> Self {
        OpError(err)
    }
}
