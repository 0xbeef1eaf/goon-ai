use deno_core::error::AnyError;
use deno_error::JsErrorClass;
use std::borrow::Cow;

#[derive(Debug)]
pub struct OpError(pub AnyError);

impl OpError {
    pub fn new(msg: &str) -> Self {
        OpError(anyhow::anyhow!("{}", msg))
    }
}

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
    fn get_additional_properties(
        &self,
    ) -> Box<dyn Iterator<Item = (Cow<'static, str>, deno_error::PropertyValue)> + 'static> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_op_error_display() {
        let err = anyhow!("Something went wrong");
        let op_err = OpError::from(err);
        assert_eq!(format!("{}", op_err), "Something went wrong");
    }

    #[test]
    fn test_op_error_js_class() {
        let err = anyhow!("Error");
        let op_err = OpError::from(err);
        assert_eq!(op_err.get_class(), "Error");
        assert_eq!(op_err.get_message(), "Error");
    }
}
