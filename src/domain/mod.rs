pub mod subscriber_email;
pub mod subscriber_name;

pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;

#[derive(Debug)]
pub struct Subscriber {
    name: SubscriberName,
    email: SubscriberEmail,
}

impl Subscriber {
    pub fn new(name: String, email: String) -> Result<Self, ()> {
        Ok(Subscriber {
            name: SubscriberName::parse(name)?,
            email: SubscriberEmail::parse(email)?,
        })
    }

    pub fn name(&self) -> &str {
        &self.name.as_ref()
    }

    pub fn email(&self) -> &str {
        &self.email.as_ref()
    }
}
