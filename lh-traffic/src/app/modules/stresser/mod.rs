pub mod application;
pub mod domain;

// Re-exports for convenience
pub use application::stress_anti_phishing_domain::{
    StressAntiPhishingDomainInputDto,
    StressAntiPhishingDomainOutputDto,
    StressAntiPhishingDomainService,
};
