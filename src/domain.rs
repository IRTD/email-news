use unicode_segmentation::UnicodeSegmentation;

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

    pub fn inner(self) -> String {
        self.0
    }
}

pub struct Subscriber {
    name: SubscriberName,
    email: String,
}

impl Subscriber {
    pub fn new(name: String, email: String) -> Result<Self, ()> {
        Ok(Subscriber {
            name: SubscriberName::parse(name)?,
            email,
        })
    }

    pub fn name(&self) -> &str {
        &self.name.0
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}
