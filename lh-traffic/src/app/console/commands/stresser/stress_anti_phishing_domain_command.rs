use crate::app::modules::shared::infrastructure::components::cli::CliColor;
use crate::app::modules::stresser::{
    StressAntiPhishingDomainInputDto,
    StressAntiPhishingDomainOutputDto,
    StressAntiPhishingDomainService,
};
use crate::app::console::abstract_command::AbstractCommand;

/// Command to perform stress testing on anti-phishing domain API
pub struct StressAntiPhishingDomainCommand {
    base: AbstractCommand,
}

impl StressAntiPhishingDomainCommand {
    pub fn new() -> Self {
        Self {
            base: AbstractCommand::new(),
        }
    }

    pub fn get_instance() -> Self {
        Self::new()
    }

    pub async fn invoke(
        &mut self,
        api_url: Option<String>,
        device_auth_token: Option<String>,
        requests_per_second: u64,
        duration_seconds: u64,
        custom_domains: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.base.echo_start(std::any::type_name::<Self>());

        match self.execute(
            api_url,
            device_auth_token,
            requests_per_second,
            duration_seconds,
            custom_domains,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                self.base
                    .logger
                    .log_error(&format!("Error: {}", e), "StressAntiPhishingDomainCommand")
                    .await;
                CliColor::die_red(&format!("Error: {}", e));
            }
        }

        self.base.echo_end("StressAntiPhishingDomainCommand");
        Ok(())
    }

    async fn execute(
        &mut self,
        api_url: Option<String>,
        device_auth_token: Option<String>,
        requests_per_second: u64,
        duration_seconds: u64,
        custom_domains: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let stress_aph_service: StressAntiPhishingDomainService = StressAntiPhishingDomainService::get_instance();

        // Use stress_aph_service defaults if not provided
        let api_url: String = api_url.unwrap_or_else(|| stress_aph_service.get_default_api_url());
        let device_auth_token: String = device_auth_token
            .unwrap_or_else(|| stress_aph_service.get_default_device_auth_token());

        self.base.echo_step(&format!("API URL: {}", api_url));
        self.base
            .echo_step(&format!("Requests per second: {}", requests_per_second));
        self.base
            .echo_step(&format!("Duration: {} seconds", duration_seconds));
        self.base.echo_step(&format!(
            "Total requests planned: {}",
            requests_per_second * duration_seconds
        ));

        if custom_domains.is_empty() {
            self.base
                .echo_step("Using default domain list (50+ domains)");
        } else {
            self.base
                .echo_step(&format!("Using {} custom domains", custom_domains.len()));
        }

        println!();
        CliColor::echo_yellow("Starting stress test...");
        println!();

        let stress_aph_domain_input_dto: StressAntiPhishingDomainInputDto = StressAntiPhishingDomainInputDto {
            api_url,
            device_auth_token,
            requests_per_second,
            duration_seconds,
            custom_domains,
        };

        // Execute stress_aph_service (already created above)
        let output: StressAntiPhishingDomainOutputDto = stress_aph_service.invoke(stress_aph_domain_input_dto).await?;

        // Display results
        println!();
        CliColor::echo_cyan("================================================================================");
        CliColor::echo_cyan("                          STRESS TEST RESULTS");
        CliColor::echo_cyan("================================================================================");
        println!();

        CliColor::echo_green(&format!("Total Requests:        {}", output.total_requests));
        CliColor::echo_green(&format!(
            "Successful Requests:   {} ({:.2}%)",
            output.successful_requests,
            (output.successful_requests as f64 / output.total_requests as f64) * 100.0
        ));
        CliColor::echo_red(&format!(
            "Failed Requests:       {} ({:.2}%)",
            output.failed_requests,
            (output.failed_requests as f64 / output.total_requests as f64) * 100.0
        ));

        println!();
        CliColor::echo_cyan("Response Times:");
        println!("  Average:  {:.2} ms", output.avg_response_time_ms);
        println!("  Minimum:  {:.2} ms", output.min_response_time_ms);
        println!("  Maximum:  {:.2} ms", output.max_response_time_ms);

        println!();
        CliColor::echo_cyan("Performance:");
        println!(
            "  Actual RPS:       {:.2} req/s",
            output.actual_rps
        );
        println!(
            "  Total Duration:   {:.2} seconds",
            output.total_duration_seconds
        );

        println!();
        CliColor::echo_cyan("Status Code Distribution:");
        let mut status_codes: Vec<(&u16, &u64)> = output.status_codes.iter().collect();
        status_codes.sort_by_key(|(code, _)| *code);

        for (code, count) in status_codes {
            let status_label: String = match *code {
                0 => "Connection Error".to_string(),
                200..=299 => format!("{} (Success)", code),
                400..=499 => format!("{} (Client Error)", code),
                500..=599 => format!("{} (Server Error)", code),
                _ => format!("{}", code),
            };

            let percentage: f64 = (*count as f64 / output.total_requests as f64) * 100.0;

            if *code >= 200 && *code < 300 {
                CliColor::echo_green(&format!("  {:<25} {} ({:.2}%)", status_label, count, percentage));
            } else if *code == 0 || *code >= 500 {
                CliColor::echo_red(&format!("  {:<25} {} ({:.2}%)", status_label, count, percentage));
            } else {
                CliColor::echo_yellow(&format!("  {:<25} {} ({:.2}%)", status_label, count, percentage));
            }
        }

        println!();
        CliColor::echo_cyan("================================================================================");

        Ok(())
    }
}
