use lh_traffic::app::modules::networking::infrastructure::repositories::{
    SystemNetworkReaderRepository,
    IpGeolocationRepository,
    ProcessInfoRepository,
};
use colored::Colorize;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    println!("{}", "=".repeat(120).bright_blue());
    println!("{}", "ENHANCED NETWORK CONNECTIONS - WITH GEOLOCATION & PROCESS INFO".bright_cyan().bold());
    println!("{}", "=".repeat(120).bright_blue());
    println!();

    // Obtener las conexiones de red
    let system_network_reader_repository: SystemNetworkReaderRepository = SystemNetworkReaderRepository::get_instance();
    let ip_geolocation_repository: IpGeolocationRepository = IpGeolocationRepository::get_instance();
    let process_info_repository: ProcessInfoRepository = ProcessInfoRepository::get_instance();

    match system_network_reader_repository.get_local_network_traffic().await {
        Ok(connections) => {
            if connections.is_empty() {
                println!("{}", "No active network connections found.".yellow());
                return;
            }

            println!("Found {} connections. Enriching with geolocation and process info...\n",
                connections.len().to_string().bright_green().bold());

            // Contar conexiones por país (para mostrar estadísticas al final)
            let mut country_stats: HashMap<String, usize> = HashMap::new();

            // Mostrar solo las primeras 10 conexiones con información detallada
            let sample_size = std::cmp::min(10, connections.len());

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

                    // Intentar obtener información completa del proceso
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

                // Geolocalización de la IP remota
                let remote_ip = ip_geolocation_repository.extract_ip_from_address(&conn.foreign_address);

                match ip_geolocation_repository.get_geolocation(&remote_ip).await {
                    Ok(Some(ip_geolocation_info)) => {
                        println!("  {} {} ({})",
                            "Country:".bright_yellow(),
                            ip_geolocation_info.country.bright_green(),
                            ip_geolocation_info.country_code.bright_green().bold()
                        );
                        println!("  {} {} - {}",
                            "Location:".bright_yellow(),
                            ip_geolocation_info.city.bright_white(),
                            ip_geolocation_info.region.bright_black()
                        );
                        println!("  {} {}",
                            "ISP:".bright_yellow(),
                            ip_geolocation_info.isp.bright_blue()
                        );
                        if ip_geolocation_info.lat != 0.0 && ip_geolocation_info.lon != 0.0 {
                            println!("  {} {}, {}",
                                "Coordinates:".bright_yellow(),
                                ip_geolocation_info.lat.to_string().bright_black(),
                                ip_geolocation_info.lon.to_string().bright_black()
                            );
                        }

                        // Actualizar estadísticas
                        *country_stats.entry(ip_geolocation_info.country_code.clone()).or_insert(0) += 1;
                    }
                    Ok(None) => {
                        println!("  {} {}",
                            "Location:".bright_yellow(),
                            "Local/Private IP".bright_black()
                        );
                    }
                    Err(e) => {
                        println!("  {} {}",
                            "Geolocation Error:".red(),
                            e.to_string().bright_black()
                        );
                    }
                }

                println!();

                // Pausa para no exceder el rate limit de la API (45 req/min)
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
            }

            // Procesar el resto de conexiones solo para estadísticas (sin mostrar detalles)
            if connections.len() > sample_size {
                println!("{}",
                    format!("Processing remaining {} connections for statistics...",
                        connections.len() - sample_size).bright_black()
                );

                for conn in connections.iter().skip(sample_size) {
                    let remote_ip = ip_geolocation_repository.extract_ip_from_address(&conn.foreign_address);

                    if let Ok(Some(ip_geolocation_info)) = ip_geolocation_repository.get_geolocation(&remote_ip).await {
                        *country_stats.entry(ip_geolocation_info.country_code.clone()).or_insert(0) += 1;
                    }

                    // Pausa entre requests
                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                }
            }

            // Mostrar estadísticas por país
            if !country_stats.is_empty() {
                println!();
                println!("{}", "=".repeat(120).bright_blue());
                println!("{}", "STATISTICS BY COUNTRY".bright_cyan().bold());
                println!("{}", "=".repeat(120).bright_blue());
                println!();

                let mut sorted_stats: Vec<_> = country_stats.iter().collect();
                sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

                for (country_code, count) in sorted_stats {
                    let percentage = (*count as f64 / connections.len() as f64) * 100.0;
                    println!("  {} {} connections ({:.1}%)",
                        country_code.bright_green().bold(),
                        count.to_string().bright_white(),
                        percentage.to_string().bright_black()
                    );
                }
            }

            println!();
            println!("{}", "=".repeat(120).bright_blue());
            println!("{}",
                format!("Total connections analyzed: {}", connections.len()).bright_green().bold()
            );
            println!("{}", "=".repeat(120).bright_blue());
        }
        Err(e) => {
            eprintln!("{} {}", "Error getting network connections:".red(), e);
            std::process::exit(1);
        }
    }
}
