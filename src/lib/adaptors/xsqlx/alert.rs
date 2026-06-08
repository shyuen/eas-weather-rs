use crate::adaptors::xsqlx::model::DatabaseMySql;
use crate::domain::alert::model::Alert;
use crate::domain::alert::new_types::alert_identifier::AlertIdentifier;
use crate::domain::alert::new_types::alert_msg_type::AlertMsgType;
use crate::domain::alert::new_types::alert_references::AlertReferences;
use crate::domain::alert::new_types::alert_references::ExtendedMessageIdentifier;
use crate::domain::alert::new_types::alert_scope::AlertScope;
use crate::domain::alert::new_types::alert_sender::AlertSender;
use crate::domain::alert::new_types::alert_sent::AlertSent;
use crate::domain::alert::new_types::alert_source::AlertSource;
use crate::domain::alert::new_types::alert_status::AlertStatus;
use crate::domain::alert::port::AlertPort;
use crate::domain::alert::port::{
    GetDailyAlertsError, GetDailyAlertsResponse, GetLatestAlertsError, GetLatestAlertsResponse,
};

use sqlx::FromRow;
use time::OffsetDateTime;
use time::format_description::well_known::{Iso8601, iso8601, iso8601::TimePrecision};

// Create custom marco to handle format
time::serde::format_description!(
    my_iso8601_format, // Name of the generated module
    OffsetDateTime,    // The type you are serializing
    FORMAT             // The format constant defined above
);

// Define the desired format configuration
// This is because the default serde implementation for ISO 8601 uses a 6-digit year format
const FORMAT: Iso8601<
    {
        iso8601::Config::DEFAULT
            .set_year_is_six_digits(false) // Disable default 6 digit year format
            .set_time_precision(TimePrecision::Second {
                decimal_digits: None, //To specify exactly X digits (microseconds) return Some(NonZeroU8::new(X).expect("X is not zero")),
            })
            .encode()
    },
> = Iso8601;

///
#[derive(
    FromRow,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    sqlx::Type,
)]
pub struct MySqlAlert {
    pub identifier: String,
    pub sender: String,
    //pub sent: String,
    #[serde(with = "my_iso8601_format")]
    pub sent: OffsetDateTime,
    pub status: String,
    pub msgtype: String,
    pub source: Option<String>,
    pub scope: String,
    pub references: Option<String>,
}

impl AlertPort for DatabaseMySql {
    async fn get_daily_alerts_data(&self) -> Result<GetDailyAlertsResponse, GetDailyAlertsError> {
        match self.get_pool() {
            Some(pool) => {
                // Start a new transaction
                let mut tx = pool.begin().await.map_err(|e| {
                    GetDailyAlertsError::DatabaseConnectionError(format!(
                        "failed to begin database transaction: {}",
                        e
                    ))
                })?;

                // Execute the query to retrieve the latest version of each alert identifier within the last 24 hours, excluding Cancelled alerts
                match sqlx::query_as::<_, MySqlAlert>(
                    r#"
                    WITH ranked_alerts AS (
                      SELECT
                      	alert.*, ROW_NUMBER()
                      OVER (
                      	PARTITION BY
                      		`identifier`
                      	ORDER BY
                      		`sent`
                      	DESC
                      ) AS rn
                      FROM
                      	Alerts AS alert
                      WHERE
                      	`sent` >= CURDATE()
                        AND `sent` < CURDATE() + INTERVAL 1 DAY
                    )
                    SELECT
                        `identifier`,
                        `sender`,
                        `sent`,
                        `status`,
                        `msgtype`,
                        `source`,
                        `scope`,
                        `references`
                    FROM
                        ranked_alerts
                    WHERE
                        rn = 1 -- Choose the first row number which is the latest version for an alert identifier
	                    AND
	                    `msgtype` != "Cancel" -- Ignore Cancelled alerts
                    LIMIT
                        100;
                    "#,
                )
                .fetch_all(&mut *tx)
                .await
                {
                    Ok(alert_items) => {
                        //info!("successfully retrieved alert items from database");

                        // Commit the transaction
                        tx.commit().await.map_err(|e| {
                            GetDailyAlertsError::DatabaseConnectionError(format!(
                                "failed to commit database transaction: {}",
                                e
                            ))
                        })?;

                        // Convert MySqlAlert to Alert
                        let alerts: Result<Vec<_>, _> = alert_items
                            .into_iter()
                            .map(|mysql_alert| Alert::try_from(mysql_alert))
                            .collect();

                        // Handle conversion result
                        match alerts {
                            Ok(alerts) => {
                                // Return the successful response with the alerts
                                Ok(GetDailyAlertsResponse { alerts })
                            }
                            Err(errors) => {
                                let error_msg = errors.join("; ");
                                //error!("data conversion errors occurred: {}", error_msg);
                                Err(GetDailyAlertsError::DataConversionError(error_msg))
                            }
                        }
                    }
                    Err(e) => {
                        //error!("failed to retrieve alert items from database: {}", e);
                        tx.commit().await.map_err(|e| {
                            GetDailyAlertsError::DatabaseConnectionError(format!(
                                "failed to commit database transaction: {}",
                                e
                            ))
                        })?;
                        return Err(GetDailyAlertsError::DatabaseConnectionError(format!(
                            "failed to retrieve alert items: {}",
                            e
                        )));
                    }
                }
            }

            None => Err(GetDailyAlertsError::DatabaseConnectionError(
                "connection pool to MySQL is not initialized".to_string(),
            )),
        }
    }

    async fn get_latest_alerts_data(
        &self,
    ) -> Result<GetLatestAlertsResponse, GetLatestAlertsError> {
        match self.get_pool() {
            Some(pool) => {
                let mut tx = pool.begin().await.map_err(|e| {
                    GetLatestAlertsError::DatabaseConnectionError(format!(
                        "failed to begin database transaction: {}",
                        e
                    ))
                })?;

                match sqlx::query_as::<_, MySqlAlert>(
                    r#"
                    SELECT
                        `identifier`,
                        `sender`,
                        `sent`,
                        `status`,
                        `msgtype`,
                        `source`,
                        `scope`,
                        `references`
                    FROM
                        Alerts
                    ORDER BY
                        `sent` DESC
                    LIMIT
                        100
                    "#,
                )
                .fetch_all(&mut *tx)
                .await
                {
                    Ok(alert_items) => {
                        tx.commit().await.map_err(|e| {
                            GetLatestAlertsError::DatabaseConnectionError(format!(
                                "failed to commit database transaction: {}",
                                e
                            ))
                        })?;

                        let alerts: Result<Vec<_>, _> = alert_items
                            .into_iter()
                            .map(|mysql_alert| Alert::try_from(mysql_alert))
                            .collect();

                        match alerts {
                            Ok(alerts) => Ok(GetLatestAlertsResponse { alerts }),
                            Err(errors) => {
                                let error_msg = errors.join("; ");
                                Err(GetLatestAlertsError::DataConversionError(error_msg))
                            }
                        }
                    }
                    Err(e) => {
                        tx.commit().await.map_err(|e| {
                            GetLatestAlertsError::DatabaseConnectionError(format!(
                                "failed to commit database transaction: {}",
                                e
                            ))
                        })?;
                        Err(GetLatestAlertsError::DatabaseConnectionError(format!(
                            "failed to retrieve alert items: {}",
                            e
                        )))
                    }
                }
            }
            None => Err(GetLatestAlertsError::DatabaseConnectionError(
                "connection pool to MySQL is not initialized".to_string(),
            )),
        }
    }
}

/// Implement TryFrom to convert MySqlAlert to Alert, handling potential conversion errors
impl TryFrom<MySqlAlert> for Alert {
    type Error = Vec<String>;

    fn try_from(mysql_alert: MySqlAlert) -> Result<Self, Self::Error> {
        let mut errors = Vec::new();

        let identifier = AlertIdentifier::new(mysql_alert.identifier);
        if identifier.is_err() {
            errors.push(format!(
                "failed to create AlertIdentifier from MySqlAlert: {:?}",
                identifier.as_ref().err()
            ));
        }

        let sender = AlertSender::new(mysql_alert.sender);
        if sender.is_err() {
            errors.push(format!(
                "failed to create AlertSender from MySqlAlert: {:?}",
                sender.as_ref().err()
            ));
        }

        let sent = AlertSent::new(mysql_alert.sent);
        if sent.is_err() {
            errors.push(format!(
                "failed to create AlertSent from MySqlAlert: {:?}",
                sent.as_ref().err()
            ));
        }

        let status = AlertStatus::new(mysql_alert.status);
        if status.is_err() {
            errors.push(format!(
                "failed to create AlertStatus from MySqlAlert: {:?}",
                status.as_ref().err()
            ));
        }

        let msgtype = AlertMsgType::new(mysql_alert.msgtype);
        if msgtype.is_err() {
            errors.push(format!(
                "failed to create AlertMsgType from MySqlAlert: {:?}",
                msgtype.as_ref().err()
            ));
        }

        let source = AlertSource::new(&mysql_alert.source.unwrap_or_default());
        if source.is_err() {
            errors.push(format!(
                "failed to create AlertSource from MySqlAlert: {:?}",
                source.as_ref().err()
            ));
        }

        let scope = AlertScope::new(mysql_alert.scope);
        if scope.is_err() {
            errors.push(format!(
                "failed to create AlertScope from MySqlAlert: {:?}",
                scope.as_ref().err()
            ));
        }

        // Create AlertReferences from space-separated ExtendedMessageIdentifiers
        let ext_msg_idents = match &mysql_alert.references {
            Some(refs) if !refs.is_empty() => {
                let mut idents = Vec::new();
                for raw in refs.split(' ') {
                    match ExtendedMessageIdentifier::new(raw) {
                        Ok(ident) => idents.push(ident),
                        Err(e) => {
                            errors.push(format!(
                                "failed to create ExtendedMessageIdentifier: {:?}",
                                e
                            ));
                            break;
                        }
                    }
                }
                idents
            }
            _ => Vec::new(),
        };
        if !errors.is_empty() {
            return Err(errors);
        }

        let references = AlertReferences::new(ext_msg_idents);

        Ok(Alert::new(
            identifier.unwrap(),
            sender.unwrap(),
            sent.unwrap(),
            status.unwrap(),
            msgtype.unwrap(),
            source.unwrap(),
            scope.unwrap(),
            references.unwrap(),
        ))
    }
}
