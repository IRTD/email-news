use crate::domain::SubscriberEmail;

use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    sender: SubscriberEmail,
    base_url: String,
    auth_token: Secret<String>,
}

impl EmailClient {
    pub fn new(sender: SubscriberEmail, base_url: String, auth_token: Secret<String>) -> Self {
        EmailClient {
            http_client: Client::new(),
            sender,
            base_url,
            auth_token,
        }
    }

    pub async fn send(
        &self,
        recv: SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", &self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().into(),
            to: recv.as_ref().into(),
            subject,
            html_body,
            text_body,
        };

        let builder = self
            .http_client
            .post(&url)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .json(&request_body)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence, Sentences};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::Request;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, r: &Request) -> bool {
            let res: Result<serde_json::Value, _> = serde_json::from_slice(&r.body);
            if let Ok(body) = res {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_the_expected_request() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(sender, mock_server.uri(), Secret::new(Faker.fake()));

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let sub_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send(sub_email, &subject, &content, &content)
            .await;
    }
}
