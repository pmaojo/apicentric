use super::ReportingError;
use crate::domain::contract_testing::ContractValidationResult;
use crate::domain::ports::contract::{
<<<<<<< HEAD
    ContractNotificationSender, ContractReportSink, NotificationConfig, ReportFormat,
    ReportingConfig,
=======
    ContractNotificationSender, ContractReportSink, NotificationConfig, ReportFormat, ReportingConfig,
>>>>>>> origin/main
};
use tracing::{error, info};

pub struct ReportingUseCase<R: ContractReportSink> {
    report_sink: R,
    notification_sender: Box<dyn ContractNotificationSender>,
    config: ReportingConfig,
}

impl<R: ContractReportSink> ReportingUseCase<R> {
    pub fn new(
        report_sink: R,
        notification_sender: Box<dyn ContractNotificationSender>,
        config: ReportingConfig,
    ) -> Self {
        Self {
            report_sink,
            notification_sender,
            config,
        }
    }

    pub async fn publish_validation_report(
        &self,
        result: &ContractValidationResult,
    ) -> Result<ReportPublicationResult, ReportingError> {
        info!(
            "Publishing validation report for contract: {}",
            result.contract_id
        );

        let mut published_formats = Vec::new();

        if let Err(e) = self
            .report_sink
            .write_report(ReportFormat::Json, result)
            .await
        {
            error!("Failed to write JSON report: {}", e);
        } else {
            published_formats.push("JSON".to_string());
        }

        if let Err(e) = self
            .report_sink
            .write_report(ReportFormat::JUnit, result)
            .await
        {
            error!("Failed to write JUnit report: {}", e);
        } else {
            published_formats.push("JUnit".to_string());
        }

        if self.config.generate_html {
            if let Err(e) = self
                .report_sink
                .write_report(ReportFormat::Html, result)
                .await
            {
                error!("Failed to write HTML report: {}", e);
            } else {
                published_formats.push("HTML".to_string());
            }
        }

        let badge_url = if self.config.generate_badges {
            match self
                .report_sink
                .write_report(ReportFormat::Badge, result)
                .await
            {
                Ok(url) => url,
                Err(e) => {
                    error!("Failed to generate badge: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(ReportPublicationResult {
            contract_id: result.contract_id.clone(),
            published_formats,
            badge_url,
        })
    }

    pub async fn send_notifications(
        &self,
        result: &ContractValidationResult,
        notification_config: &NotificationConfig,
    ) -> Result<(), ReportingError> {
        if let Some(webhook_url) = &notification_config.slack_webhook_url {
            if let Err(e) = self
                .notification_sender
                .send_slack_notification(webhook_url, result)
                .await
            {
                error!("Failed to send Slack notification: {}", e);
            } else {
                info!(
                    "Sent Slack notification for contract: {}",
                    result.contract_id
                );
            }
        }

        if !notification_config.email_recipients.is_empty() {
            if let Err(e) = self
                .notification_sender
                .send_email_notification(&notification_config.email_recipients, result)
                .await
            {
                error!("Failed to send email notifications: {}", e);
            } else {
                info!(
                    "Sent email notifications for contract: {}",
                    result.contract_id
                );
            }
        }

        Ok(())
    }
}

pub struct ReportPublicationResult {
    pub contract_id: crate::domain::contract_testing::ContractId,
    pub published_formats: Vec<String>,
    pub badge_url: Option<String>,
}
