use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use reqwest;
use reqwest::header::CONTENT_TYPE;
use std::env;

// Simply check if we can connect to url. If not, we need to send a Slack notification.
#[tracing::instrument(skip(event), fields(req_id = %event.context.request_id))]
async fn function_handler(event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    let detail = event
        .payload
        .detail
        .expect("Payload doesn't have needed url field");

    tracing::info!("Checking health of {:?}", detail["url"]);

    // Extract some useful information from the request
    let resp = reqwest::get(detail["url"].to_string()).await?;
    if resp.status().is_success() {
        tracing::info!("Success!");
    } else if resp.status().is_server_error() {
        tracing::info!("Server error!");
        send_slack_notification(&resp.text().await.unwrap()).await;
    } else {
        tracing::info!("Something else happened. Status: {:?}", resp.status());
    }

    Ok(())
}

async fn send_slack_notification(msg: &str) {
    let token =
        env::var("SLACK_BOT_TOKEN")
        .expect("slack bot token is not set.");
    
    let params = [("token", token), ("channel", String::from("notifications")), ("text", String::from(msg))];
    let client = reqwest::Client::new();
    let _ = client.post("https://slack.com/api/chat.postMessage")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .expect("Slack API call error");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
