use url::Url;

/// Valid link prefix donor for the RSS channel item
pub struct Link(String);

impl Link {
    pub fn from_url(canonical: Option<Url>) -> Self {
        Self(
            canonical
                .map(|mut c| {
                    c.set_path("/");
                    c.set_fragment(None);
                    c.set_query(None);
                    super::escape(c.as_str()) // filter once
                })
                .unwrap_or_default(), // should be non-optional absolute URL
                                      // by the RSS specification @TODO
        )
    }
    pub fn link(&self, id: i64) -> String {
        format!("{}{id}", self.0)
    }
}
