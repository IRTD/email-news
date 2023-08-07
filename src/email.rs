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
    pub fn new(
        sender: SubscriberEmail,
        base_url: String,
        auth_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = reqwest::ClientBuilder::new()
            .timeout(timeout)
            .build()
            .unwrap();

        EmailClient {
            http_client,
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
            .await?
            .error_for_status()?;

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
    use wiremock::matchers::{any, header, header_exists, method, path};
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

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            email(),
            base_url,
            Secret::new(Faker.fake()),
            std::time::Duration::from_secs(10),
        )
    }

    #[tokio::test]
    async fn send_email_the_expected_request_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let sub_email = email();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let res = email_client
            .send(sub_email, &subject, &content, &content)
            .await;

        claims::assert_ok!(res);
    }

    #[tokio::test]
    async fn send_email_the_expected_request_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let sub_email = email();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let res = email_client
            .send(sub_email, &subject, &content, &content)
            .await;

        claims::assert_err!(res);
    }

    #[tokio::test]
    async fn send_email_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(60));
        Mock::given(any())
            .and(SendEmailBodyMatcher)
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let sub_email = email();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let res = email_client
            .send(sub_email, &subject, &content, &content)
            .await;

        claims::assert_err!(res);
    }
}
