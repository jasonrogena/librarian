use crate::error;
use std::collections;

#[cfg(test)]
mod tests;

pub struct Template {
    tmpl: String,
}

impl Template {
    pub fn new(tmpl: String) -> Result<Template, error::Error> {
        Ok(Template { tmpl })
    }

    pub fn render(&self, data: &collections::HashMap<&str, &str>) -> Result<String, error::Error> {
        let mut context = tera::Context::new();
        for (cur_key, cur_val) in data {
            context.insert(*cur_key, *cur_val);
        }

        match tera::Tera::one_off(self.tmpl.as_str(), &context, false) {
            Err(e) => Err(error::Error::new(format!(
                "Could not render template: {:?}",
                e
            ))),
            Ok(s) => Ok(s),
        }
    }
}
