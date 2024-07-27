use std::collections;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A Tera templating error occurred")]
    Io(#[from] tera::Error),
}

pub struct Template {
    tmpl: String,
}

impl Template {
    pub fn new(tmpl: String) -> Result<Template, Error> {
        Ok(Template { tmpl })
    }

    pub fn render(&self, data: &collections::HashMap<&str, &str>) -> Result<String, Error> {
        let mut context = tera::Context::new();
        for (cur_key, cur_val) in data {
            context.insert(*cur_key, *cur_val);
        }

        Ok(tera::Tera::one_off(self.tmpl.as_str(), &context, false)?)
    }
}
