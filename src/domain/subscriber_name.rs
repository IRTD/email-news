use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

// Replace '()' with actual error to report details back to the user
impl SubscriberName {
    pub fn parse(name: String) -> Result<SubscriberName, ()> {
        let is_empty_or_whitespace = name.trim().is_empty();
        let is_too_long = name.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '{', '}', '\\'];
        let contains_forbidden_character = name.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_character {
            return Err(());
        } else {
            return Ok(Self(name));
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;

    #[test]
    fn a_256_grapheme_name_accepted() {
        let name = "e".repeat(256);
        claims::assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn longer_than_256_fails() {
        let name = "e".repeat(257);
        claims::assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn only_whitespace_name_fails() {
        let name = " ".to_string();
        claims::assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_name_fails() {
        let name = "".to_string();
        claims::assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn invalid_character_fails() {
        for c in ['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = c.to_string();
            claims::assert_err!(SubscriberName::parse(name))
        }
    }

    #[test]
    fn valid_name_accepted() {
        let name = "Marie".to_string();
        claims::assert_ok!(SubscriberName::parse(name));
    }
}
