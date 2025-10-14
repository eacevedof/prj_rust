use anyhow::{Result, Context};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio::task::JoinHandle;
use rand::seq::SliceRandom;

use crate::{log_info, log_warn, log_error};

use super::stress_anti_phishing_domain_input_dto::StressAntiPhishingDomainInputDto;
use super::stress_anti_phishing_domain_output_dto::StressAntiPhishingDomainOutputDto;

/// Service for stress testing the anti-phishing domain API
pub struct StressAntiPhishingDomainService {
    default_api_url: String,
    default_device_auth_token: String,
}

impl StressAntiPhishingDomainService {
    /// Create a new instance with default configuration
    pub fn new() -> Self {
        Self {
            default_api_url: "https://app-ms-antiphising.deno.dev/api/v1/anti-phising/domain".to_string(),
            default_device_auth_token: "aph-dev-auth-iWkAeTMtU0znGOItSmZvmvcxFzlI60I3HOW".to_string(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    /// Get default API URL
    pub fn get_default_api_url(&self) -> String {
        self.default_api_url.clone()
    }

    /// Get default device auth token
    pub fn get_default_device_auth_token(&self) -> String {
        self.default_device_auth_token.clone()
    }

    /// Execute stress test
    pub async fn invoke(
        &self,
        input: StressAntiPhishingDomainInputDto,
    ) -> Result<StressAntiPhishingDomainOutputDto> {
        log_info!(
            "Starting stress test: {} req/s for {} seconds",
            input.requests_per_second,
            input.duration_seconds
        );

        let domains: Vec<String> = if input.custom_domains.is_empty() {
            Self::get_default_domains()
        } else {
            input.custom_domains.clone()
        };

        log_info!("Using {} domains for testing", domains.len());

        // Shared state for results
        let results: Arc<Mutex<StressResults>> = Arc::new(Mutex::new(StressResults::new()));
        let start_time: Instant = Instant::now();

        // Calculate total requests
        let total_requests: u64 = input.requests_per_second * input.duration_seconds;
        let delay_between_requests: Duration = Duration::from_millis(1000 / input.requests_per_second);

        log_info!("Total requests to send: {}", total_requests);
        log_info!("Delay between requests: {:?}", delay_between_requests);

        // Create HTTP client (reuse connection pool)
        let request_client: Client = reqwest::Client::builder()
            .pool_max_idle_per_host(input.requests_per_second as usize)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP request_client")?;

        let request_client: Arc<Client> = Arc::new(request_client);

        let mut tasks: Vec<JoinHandle<()>>= Vec::new();

        for i in 0..total_requests {
            let client: Arc<Client> = request_client.clone();
            let results: Arc<Mutex<StressResults>> = results.clone();
            let api_url: String = input.api_url.clone();
            let device_auth_token: String = input.device_auth_token.clone();
            let domains: Vec<String> = domains.clone();

            // Spawn task for each request
            let task: tokio::task::JoinHandle<()> = tokio::spawn(async move {
                // Select random domain
                let domain: String = domains.choose(&mut rand::thread_rng()).unwrap().clone();
                let domain_uuid: String = Self::generate_md5(&domain);

                // Create request body
                let body: serde_json::Value = serde_json::json!({
                    "domain_uuid": domain_uuid,
                    "domain": domain,
                });

                // Measure request time
                let req_start: Instant = Instant::now();

                let response: Result<reqwest::Response, reqwest::Error> = client
                    .post(&api_url)
                    .header("lzrmsaph-device-auth", &device_auth_token)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await;

                let elapsed: Duration = req_start.elapsed();

                // Update results
                let mut results: tokio::sync::MutexGuard<StressResults> = results.lock().await;
                results.total_requests += 1;

                match response {
                    Ok(resp) => {
                        let status: u16 = resp.status().as_u16();
                        *results.status_codes.entry(status).or_insert(0) += 1;

                        if resp.status().is_success() {
                            results.successful_requests += 1;
                        } else {
                            results.failed_requests += 1;
                            log_warn!("Request {} failed with status: {}", i, status);
                        }

                        results.response_times.push(elapsed.as_millis() as f64);
                    }
                    Err(e) => {
                        results.failed_requests += 1;
                        log_error!("Request {} error: {}", i, e);
                        *results.status_codes.entry(0).or_insert(0) += 1; // 0 for connection errors
                    }
                }
            });

            tasks.push(task);

            // Rate limiting - wait before next request
            if (i + 1) % input.requests_per_second == 0 {
                sleep(Duration::from_secs(1)).await;
            } 
            else {
                sleep(delay_between_requests).await;
            }
        }

        // Wait for all tasks to complete
        log_info!("Waiting for all requests to complete...");
        for task in tasks {
            let _ = task.await;
        }

        let total_duration: Duration = start_time.elapsed();

        // Build output
        let results: tokio::sync::MutexGuard<StressResults> = results.lock().await;
        let output: StressAntiPhishingDomainOutputDto = self.build_output(&results, total_duration);

        log_info!("Stress test completed!");
        log_info!("Total requests: {}", output.total_requests);
        log_info!("Successful: {}", output.successful_requests);
        log_info!("Failed: {}", output.failed_requests);
        log_info!("Avg response time: {:.2}ms", output.avg_response_time_ms);
        log_info!("Actual RPS: {:.2}", output.actual_rps);

        Ok(output)
    }

    /// Build output DTO from results
    fn build_output(&self, results: &StressResults, duration: Duration) -> StressAntiPhishingDomainOutputDto {
        let avg_response_time: f64 = if !results.response_times.is_empty() {
            results.response_times.iter().sum::<f64>() / results.response_times.len() as f64
        } else {
            0.0
        };

        let min_response_time: f64 = results
            .response_times
            .iter()
            .cloned()
            .fold(f64::MAX, f64::min);

        let max_response_time: f64 = results
            .response_times
            .iter()
            .cloned()
            .fold(0.0, f64::max);

        let actual_rps: f64 = results.total_requests as f64 / duration.as_secs_f64();

        StressAntiPhishingDomainOutputDto {
            total_requests: results.total_requests,
            successful_requests: results.successful_requests,
            failed_requests: results.failed_requests,
            avg_response_time_ms: avg_response_time,
            min_response_time_ms: if min_response_time == f64::MAX {
                0.0
            } else {
                min_response_time
            },
            max_response_time_ms: max_response_time,
            actual_rps,
            total_duration_seconds: duration.as_secs_f64(),
            status_codes: results.status_codes.clone(),
        }
    }

    /// Generate MD5 hash of domain (for domain_uuid)
    fn generate_md5(text: &str) -> String {
        let digest: md5::Digest = md5::compute(text.as_bytes());
        format!("{:x}", digest)
    }

    /// Get default list of domains for testing
    fn get_default_domains() -> Vec<String> {
        vec![
            "marca.es".to_string(),
            "bancosantander.es".to_string(),
            "elpais.es".to_string(),
            "elmudo.es".to_string(),
            
            "0-amazon.weebly.com".to_string(),
            "0-amfc.firebaseapp.com".to_string(),
            "0-whatsapp.com".to_string(),
            "0-x7r58fdu8f8dfe8rc.000webhostapp.com".to_string(),
            "0.0.0.0forum.cryptonight.net".to_string(),
            "0.0.0.0mailgate.cryptonight.net".to_string(),
            "0.0.0.0ns10.cryptonight.net".to_string(),
            "0.0.0.0ssl.cryptonight.net".to_string(),
            "0.0.0assets.cryptonight.net".to_string(),
            "0.0.0dbs.cryptonight.net".to_string(),
            "0.0.0fileserver.cryptonight.net".to_string(),
            "0.0.0mail3.cryptonight.net".to_string(),
            "0.0.0ns6.cryptonight.net".to_string(),
            "0.0.0t.cryptonight.net".to_string(),
            "0.0.0wsus.cryptonight.net".to_string(),
            "0.00000.life".to_string(),
            "0.0l.cryptonight.net".to_string(),
            "0.0mx03.cryptonight.net".to_string(),
            "0.232.205.92.host.secureserver.net".to_string(),
            "0.9.0.0.9.theballoonsquad.co.uk".to_string(),
            "0.bgeneral0.repl.co".to_string(),
            "0.fascinatingsciencemag.com".to_string(),
            "0.feixue316p.cloudns.biz".to_string(),
            "0.feixue317p.cloudns.biz".to_string(),
            "0.feixue318p.cloudns.biz".to_string(),
            "0.fres-news.com".to_string(),
            "0.oswqa.today".to_string(),
            "0.sciiaawuroyagu9524.workers.dev".to_string(),
            "0.tecnodentalrd.com".to_string(),
            "0.velocitas.ng".to_string(),
            "0.vsxmfkrvoipzs67.workers.dev".to_string(),
            "0.xyasrhsju3235.workers.dev".to_string(),
            "00-5fu.pages.dev".to_string(),
            "00-663b30.ingress-daribow.ewp.live".to_string(),
            "00-974a.duckdns.org".to_string(),
            "00-c1aimn0w.com".to_string(),
            "00-coveriy-tm.quest".to_string(),
            "00-dnfjg7w8er7ufxdj.000webhostapp.com".to_string(),
            "00-zinox007-yandex-999-dhl-com-002900-700-3-smart-oryx-ex.mybluemix.net".to_string(),
            "00.buh30-00.ru".to_string(),
            "00.xyz-yt.biz.id".to_string(),
            "000-00-000-000000.pages.dev".to_string(),
            "000-7yt65t564656ythygy.000webhostapp.com".to_string(),
            "000-845int283-000.xyz".to_string(),
            "000-hidro-1.info".to_string(),
            "000.abreubueno91.repl.co".to_string(),
            "0000-1t8.pages.dev".to_string(),
            "0000.com.my".to_string(),
            "0000.hopto.org".to_string(),
            "00000--bancogeneral3.repl.co".to_string(),
        ]
    }
}

/// Internal struct for collecting results
#[derive(Debug)]
struct StressResults {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    response_times: Vec<f64>,
    status_codes: std::collections::HashMap<u16, u64>,
}

impl StressResults {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            response_times: Vec::new(),
            status_codes: std::collections::HashMap::new(),
        }
    }
}
