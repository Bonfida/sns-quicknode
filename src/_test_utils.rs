use std::{
    fs::File,
    io::{BufWriter, Write},
};

use reqwest::header::HeaderMap;
use serde::Serialize;
use serde_json::Value;

use crate::sns::Method;

pub struct TestClient {
    inner: reqwest::Client,
    endpoint: String,
}

impl Default for TestClient {
    fn default() -> Self {
        #[cfg(not(feature = "test-bypass-proxy"))]
        const ENDPOINT_VAR: &str = "TEST_QUICKNODE_ENDPOINT";
        #[cfg(feature = "test-bypass-proxy")]
        const ENDPOINT_VAR: &str = "RAW_API_ENDPOINT";
        let endpoint = std::env::var(ENDPOINT_VAR).unwrap();
        #[allow(unused_mut)]
        let mut headers = HeaderMap::new();
        #[cfg(feature = "test-bypass-proxy")]
        {
            headers.insert(
                "x-quicknode-id",
                std::env::var("TEST_QUICKNODE_ID").unwrap().parse().unwrap(),
            );
            headers.insert(
                "x-instance-id",
                std::env::var("TEST_QUICKNODE_ENDPOINT_ID")
                    .unwrap()
                    .parse()
                    .unwrap(),
            );
        }
        let inner = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self { inner, endpoint }
    }
}

impl TestClient {
    pub async fn run_request<V: Serialize>(
        &self,
        message: &V,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let request = self
            .inner
            .post(&self.endpoint)
            .json(message)
            .build()
            .unwrap();
        self.inner.execute(request).await
    }
}

pub fn write_example<V1: Serialize, V2: Serialize>(method: Method, request: V1, result: V2) {
    let mut out_file = BufWriter::new(
        File::create(format!(
            "./examples/{}.md",
            serde_json::to_string(&method).unwrap().trim_matches('"')
        ))
        .unwrap(),
    );
    out_file.write_all(b"## Example\n\n```json\n").unwrap();
    serde_json::to_writer_pretty(&mut out_file, &request).unwrap();
    out_file.write_all(b"```\n\n```json\n").unwrap();
    serde_json::to_writer_pretty(&mut out_file, &result).unwrap();
    out_file.write_all(b"```\n").unwrap();
}
