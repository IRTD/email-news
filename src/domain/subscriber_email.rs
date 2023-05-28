use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<Self, ()> {
        match validate_email(&email) {
            true => Ok(Self(email)),
            false => Err(()),
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[test]
    fn empty_email_fails() {
        let email = "".to_string();
        claims::assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_at_symbol_fails() {
        let email = "mariegmail.com".to_string();
        claims::assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_subject_fails() {
        let email = "@gmail.com".to_string();
        claims::assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_email_succeeds() {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let email = SafeEmail().fake_with_rng(&mut rng);
            claims::assert_ok!(SubscriberEmail::parse(email));
        }
    }
}
