use lh_traffic::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    WhoisRepository,
    ProcessInfoRepository,
};
use colored::Colorize;

#[tokio::main]
async fn main() {
    println!("{}", "=".repeat(120).bright_blue());
    println!("{}", "NETWORK CONNECTIONS - WITH WHOIS INFO".bright_cyan().bold());
    println!("{}", "=".repeat(120).bright_blue());
    println!();

    // Obtener las conexiones de red
    let system_network_reader_repository: SystemNetworkReaderRepository = SystemNetworkReaderRepository::get_instance();
    let whois_repository: WhoisRepository = WhoisRepository::get_instance();
    let process_info_repository: ProcessInfoRepository = ProcessInfoRepository::get_instance();

    match system_network_reader_repository.get_local_network_traffic().await {
        Ok(connections) => {
            if connections.is_empty() {
                println!("{}", "No active network connections found.".yellow());
                return;
            }

            println!("Found {} connections. Getting whois info...\n",
                connections.len().to_string().bright_green().bold());

            // Mostrar solo las primeras 5 conexiones con información detallada
            // (whois es más lento que ip-api, 2-5 segundos por consulta)
            let sample_size = std::cmp::min(5, connections.len());

            for (idx, conn) in connections.iter().take(sample_size).enumerate() {
                println!("{} {}",
                    format!("Connection #{}", idx + 1).bright_white().bold(),
                    format!("({})", conn.protocol).bright_black()
                );

                // Información básica
                println!("  {} {} {} {}",
                    "Local:".bright_yellow(),
                    conn.local_address.bright_white(),
                    "->".bright_black(),
                    conn.foreign_address.bright_white()
                );
                println!("  {} {}",
                    "State:".bright_yellow(),
                    conn.state.bright_green()
                );

                // Información del proceso
                if let Some(pid) = conn.pid {
                    println!("  {} {}",
                        "PID:".bright_yellow(),
                        pid.to_string().bright_cyan()
                    );

                    match process_info_repository.get_process_info(pid).await {
                        Ok(Some(process_info)) => {
                            println!("  {} {}",
                                "Process:".bright_yellow(),
                                process_info.name.bright_magenta()
                            );
                            if !process_info.path.is_empty() && process_info.path != "Unknown" {
                                println!("  {} {}",
                                    "Path:".bright_yellow(),
                                    process_info.path.bright_black()
                                );
                            }
                            if let Some(owner) = process_info.owner {
                                println!("  {} {}",
                                    "Owner:".bright_yellow(),
                                    owner.bright_blue()
                                );
                            }
                        }
                        Ok(None) => {
                            if let Some(name) = &conn.program_name {
                                println!("  {} {}",
                                    "Process:".bright_yellow(),
                                    name.bright_black()
                                );
                            }
                        }
                        Err(e) => {
                            println!("  {} {}",
                                "Process Error:".red(),
                                e.to_string().bright_black()
                            );
                        }
                    }
                }

                // Whois de la IP remota
                let remote_ip = whois_repository.extract_ip_from_address(&conn.foreign_address);

                println!("  {} Querying whois for {}...",
                    "Whois:".bright_yellow(),
                    remote_ip.bright_cyan()
                );

                match whois_repository.get_whois_info(&remote_ip).await {
                    Ok(Some(whois_info)) => {
                        if let Some(org) = whois_info.organization {
                            println!("    {} {}",
                                "Organization:".bright_green(),
                                org.bright_white()
                            );
                        }
                        if let Some(country) = whois_info.country {
                            println!("    {} {}",
                                "Country:".bright_green(),
                                country.bright_white()
                            );
                        }
                        if let Some(net_name) = whois_info.net_name {
                            println!("    {} {}",
                                "Network:".bright_green(),
                                net_name.bright_white()
                            );
                        }
                        if let Some(ip_range) = whois_info.ip_range {
                            println!("    {} {}",
                                "IP Range:".bright_green(),
                                ip_range.bright_black()
                            );
                        }
                        if let Some(asn) = whois_info.asn {
                            print!("    {} {}",
                                "ASN:".bright_green(),
                                asn.bright_white()
                            );
                            if let Some(desc) = whois_info.asn_description {
                                print!(" ({})", desc.bright_black());
                            }
                            println!();
                        }
                    }
                    Ok(None) => {
                        println!("    {}",
                            "Local/Private IP - No whois info".bright_black()
                        );
                    }
                    Err(e) => {
                        println!("    {} {}",
                            "Error:".red(),
                            e.to_string().bright_black()
                        );
                    }
                }

                println!();
            }

            if connections.len() > sample_size {
                println!("{}",
                    format!("Showing {} of {} connections. Run 'make run-list' to see all.",
                        sample_size, connections.len()).bright_black()
                );
            }

            println!();
            println!("{}", "=".repeat(120).bright_blue());
            println!("{}",
                "NOTE: whois is slower than ip-api (2-5s per query) but has no rate limit.".bright_black()
            );
            println!("{}",
                "For faster results with geolocation, use 'make run-enhanced' instead.".bright_black()
            );
            println!("{}", "=".repeat(120).bright_blue());
        }
        Err(e) => {
            eprintln!("{} {}", "Error getting network connections:".red(), e);
            std::process::exit(1);
        }
    }
}
