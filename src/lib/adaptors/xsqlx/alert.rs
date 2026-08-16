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
    CreateAlertError, CreateAlertResponse, GetAlertError, GetAlertResponse, GetDailyAlertsError,
    GetDailyAlertsResponse, GetLatestAlertsError, GetLatestAlertsResponse, UpdateAlertError,
    UpdateAlertResponse,
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

/// A database row representing an alert, mapped from the MySQL alerts table.
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
    async fn get_daily_alerts_data(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<GetDailyAlertsResponse, GetDailyAlertsError> {
        match self.get_pool() {
            Some(pool) => {
                let mut tx = pool.begin().await.map_err(|e| {
                    GetDailyAlertsError::DatabaseConnectionError(format!(
                        "failed to begin database transaction: {}",
                        e
                    ))
                })?;

                let total: (i64,) = sqlx::query_as(
                    r#"
                    SELECT COUNT(*) FROM (
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
                    ) AS ranked
                    WHERE
                        rn = 1
                        AND
                        `msgtype` != "Cancel"
                    "#,
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    GetDailyAlertsError::DatabaseConnectionError(format!(
                        "failed to count daily alert items: {}",
                        e
                    ))
                })?;

                let alert_items = sqlx::query_as::<_, MySqlAlert>(
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
                        rn = 1
                        AND
                        `msgtype` != "Cancel"
                    ORDER BY
                        `sent` DESC
                    LIMIT
                        ?
                    OFFSET
                        ?
                    "#,
                )
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| {
                    GetDailyAlertsError::DatabaseConnectionError(format!(
                        "failed to retrieve daily alert items: {}",
                        e
                    ))
                })?;

                tx.commit().await.map_err(|e| {
                    GetDailyAlertsError::DatabaseConnectionError(format!(
                        "failed to commit database transaction: {}",
                        e
                    ))
                })?;

                let alerts: Result<Vec<_>, _> =
                    alert_items.into_iter().map(Alert::try_from).collect();

                match alerts {
                    Ok(alerts) => Ok(GetDailyAlertsResponse {
                        total: total.0 as u64,
                        alerts,
                    }),
                    Err(errors) => {
                        let error_msg = errors.join("; ");
                        Err(GetDailyAlertsError::DataConversionError(error_msg))
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
        limit: u64,
        offset: u64,
    ) -> Result<GetLatestAlertsResponse, GetLatestAlertsError> {
        match self.get_pool() {
            Some(pool) => {
                let mut tx = pool.begin().await.map_err(|e| {
                    GetLatestAlertsError::DatabaseConnectionError(format!(
                        "failed to begin database transaction: {}",
                        e
                    ))
                })?;

                let total: (i64,) = sqlx::query_as(
                    r#"
                    SELECT COUNT(*) FROM (
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
                    ) AS ranked
                    WHERE
                        rn = 1
                        AND
                        `msgtype` != "Cancel"
                    "#,
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| {
                    GetLatestAlertsError::DatabaseConnectionError(format!(
                        "failed to count alert items: {}",
                        e
                    ))
                })?;

                let alert_items = sqlx::query_as::<_, MySqlAlert>(
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
                        rn = 1
                        AND
                        `msgtype` != "Cancel"
                    ORDER BY
                        `sent` DESC
                    LIMIT
                        ?
                    OFFSET
                        ?
                    "#,
                )
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| {
                    GetLatestAlertsError::DatabaseConnectionError(format!(
                        "failed to retrieve alert items: {}",
                        e
                    ))
                })?;

                tx.commit().await.map_err(|e| {
                    GetLatestAlertsError::DatabaseConnectionError(format!(
                        "failed to commit database transaction: {}",
                        e
                    ))
                })?;

                let alerts: Result<Vec<_>, _> =
                    alert_items.into_iter().map(Alert::try_from).collect();

                match alerts {
                    Ok(alerts) => Ok(GetLatestAlertsResponse {
                        total: total.0 as u64,
                        alerts,
                    }),
                    Err(errors) => {
                        let error_msg = errors.join("; ");
                        Err(GetLatestAlertsError::DataConversionError(error_msg))
                    }
                }
            }
            None => Err(GetLatestAlertsError::DatabaseConnectionError(
                "connection pool to MySQL is not initialized".to_string(),
            )),
        }
    }
    async fn create_alert_data(
        &self,
        alert: Alert,
    ) -> Result<CreateAlertResponse, CreateAlertError> {
        match self.get_pool() {
            Some(pool) => {
                let row = MySqlAlert::from(&alert);

                let mut tx = pool.begin().await.map_err(|e| {
                    CreateAlertError::DatabaseConnectionError(format!(
                        "failed to begin database transaction: {}",
                        e
                    ))
                })?;

                sqlx::query(
                    r#"
                    INSERT INTO Alerts
                        (`identifier`, `sender`, `sent`, `status`, `msgtype`, `source`, `scope`, `references`)
                    VALUES
                        (?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&row.identifier)
                .bind(&row.sender)
                .bind(row.sent)
                .bind(&row.status)
                .bind(&row.msgtype)
                .bind(&row.source)
                .bind(&row.scope)
                .bind(&row.references)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    CreateAlertError::DatabaseError(format!(
                        "failed to insert alert: {}",
                        e
                    ))
                })?;

                tx.commit().await.map_err(|e| {
                    CreateAlertError::DatabaseConnectionError(format!(
                        "failed to commit database transaction: {}",
                        e
                    ))
                })?;

                Ok(CreateAlertResponse { alert })
            }

            None => Err(CreateAlertError::DatabaseConnectionError(
                "connection pool to MySQL is not initialized".to_string(),
            )),
        }
    }

    async fn update_alert_data(
        &self,
        identifier: &AlertIdentifier,
        alert: Alert,
    ) -> Result<UpdateAlertResponse, UpdateAlertError> {
        match self.get_pool() {
            Some(pool) => {
                let row = MySqlAlert::from(&alert);

                let mut tx = pool.begin().await.map_err(|e| {
                    UpdateAlertError::DatabaseConnectionError(format!(
                        "failed to begin database transaction: {}",
                        e
                    ))
                })?;

                let result = sqlx::query(
                    r#"
                    UPDATE Alerts
                    SET
                        `sender` = ?,
                        `sent` = ?,
                        `status` = ?,
                        `msgtype` = ?,
                        `source` = ?,
                        `scope` = ?,
                        `references` = ?
                    WHERE
                        `identifier` = ?
                    "#,
                )
                .bind(&row.sender)
                .bind(row.sent)
                .bind(&row.status)
                .bind(&row.msgtype)
                .bind(&row.source)
                .bind(&row.scope)
                .bind(&row.references)
                .bind(identifier.as_str())
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    UpdateAlertError::DatabaseError(format!("failed to update alert: {}", e))
                })?;

                if result.rows_affected() == 0 {
                    return Err(UpdateAlertError::NotFound);
                }

                tx.commit().await.map_err(|e| {
                    UpdateAlertError::DatabaseConnectionError(format!(
                        "failed to commit database transaction: {}",
                        e
                    ))
                })?;

                Ok(UpdateAlertResponse { alert })
            }

            None => Err(UpdateAlertError::DatabaseConnectionError(
                "connection pool to MySQL is not initialized".to_string(),
            )),
        }
    }

    async fn get_alert_data(
        &self,
        identifier: &AlertIdentifier,
    ) -> Result<GetAlertResponse, GetAlertError> {
        match self.get_pool() {
            Some(pool) => {
                let alert_item = sqlx::query_as::<_, MySqlAlert>(
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
                        `identifier` = ?
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
                        rn = 1
                    "#,
                )
                .bind(identifier.as_str())
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    GetAlertError::DatabaseConnectionError(format!(
                        "failed to retrieve alert: {}",
                        e
                    ))
                })?;

                match alert_item {
                    Some(item) => match Alert::try_from(item) {
                        Ok(alert) => Ok(GetAlertResponse { alert }),
                        Err(errors) => Err(GetAlertError::DataConversionError(errors.join("; "))),
                    },
                    None => Err(GetAlertError::NotFound),
                }
            }

            None => Err(GetAlertError::DatabaseConnectionError(
                "connection pool to MySQL is not initialized".to_string(),
            )),
        }
    }
}

/// Implement From to convert Alert to MySqlAlert for persistence.
impl From<&Alert> for MySqlAlert {
    fn from(alert: &Alert) -> Self {
        MySqlAlert {
            identifier: alert.identifier().as_str().to_string(),
            sender: alert.sender().as_str().to_string(),
            sent: alert.sent().as_offset_date_time(),
            status: alert.status().as_str().to_string(),
            msgtype: alert.msg_type().as_str().to_string(),
            source: alert.source().as_opt_str().map(|s| s.to_string()),
            scope: alert.scope().as_str().to_string(),
            references: alert.references().as_db_string(),
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
