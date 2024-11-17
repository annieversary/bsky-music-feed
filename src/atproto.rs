#[derive(Debug, Clone)]
pub struct AtUri<'a> {
    pub did: &'a str,
    pub collection: &'a str,
    pub rkey: &'a str,
}

impl<'a> AtUri<'a> {
    pub fn from_str(s: &'a str) -> Result<AtUri<'a>, &'static str> {
        let parts = s
            .strip_prefix("at://")
            .ok_or(r#"record uri must start with "at://""#)?
            .splitn(3, '/')
            .collect::<Vec<_>>();

        if !parts[0].starts_with("did:plc:") {
            return Err(r#"record uri must start with "at://did:plc:""#);
        }

        Ok(Self {
            did: parts[0],
            collection: parts[1],
            rkey: parts[2],
        })
    }
}

impl std::fmt::Display for AtUri<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "at://{}/{}/{}", self.did, self.collection, self.rkey)
    }
}
