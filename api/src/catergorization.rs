use crate::transaction::Transaction;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Error,
};

use reqwest::Client;

use lazy_static::lazy_static;

const API_ENDPOINT: &'static str = "https://app.fina.money/api/resource/categorize";

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub async fn catergorize_transactions(transaction: &[Transaction]) -> Result<Vec<String>, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert("x-api-key", "fina-api-test".parse().unwrap()); //api key is free and open to everyone
    headers.insert("x-partner-id", "<client-parter-id>".parse().unwrap());
    headers.insert("x-api-model", "v3".parse().unwrap());
    headers.insert("x-api-mapping", "true".parse().unwrap());

    let data: Vec<serde_json::Value> = transaction
        .iter()
        .map(|t| t.seriazlize_to_catergorize())
        .collect();

    let mut all_responses = vec![];
    for chunk in data.chunks(100) {
        let response = CLIENT
            .post(API_ENDPOINT.to_string())
            .headers(headers.clone())
            .json(&chunk)
            .send()
            .await
            .unwrap();
        let response_data = response.json::<Vec<String>>().await.unwrap();
        all_responses.extend(response_data);
    }

    Ok(all_responses)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::transaction::Transaction;

    #[tokio::test]
    async fn test_catergorize_transaction() {
        let transaction = vec![Transaction::dummy(), Transaction::dummy()];
        let result = catergorize_transactions(&transaction).await;
        println!("result: {:#?}", result);
        assert!(result.is_ok());
    }
}
