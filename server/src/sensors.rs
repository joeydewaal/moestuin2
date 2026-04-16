use std::time::Duration;

use jiff::Timestamp;
use rand::{RngExt, rng};
use toasty::Db;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    readings::Reading,
};

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DriverKind {
    Real,
    Mock,
}

pub enum SensorDriver {
    Mock(MockDriver),
}

pub struct MockDriver;

#[derive(Debug, Clone)]
pub struct SensorSample {
    pub temp_c: f64,
    pub humidity: f64,
    pub moisture: f64,
}

impl SensorDriver {
    pub fn kind(&self) -> DriverKind {
        match self {
            Self::Mock(_) => DriverKind::Mock,
        }
    }

    pub async fn read(&mut self) -> AppResult<SensorSample> {
        match self {
            Self::Mock(m) => m.read().await,
        }
    }
}

impl MockDriver {
    pub async fn read(&mut self) -> AppResult<SensorSample> {
        let mut r = rng();
        Ok(SensorSample {
            temp_c: 18.0 + r.random::<f64>() * 8.0,
            humidity: 55.0 + r.random::<f64>() * 15.0,
            moisture: 0.30 + r.random::<f64>() * 0.30,
        })
    }
}

pub async fn probe(mock_hw: bool) -> SensorDriver {
    if mock_hw {
        tracing::info!("MOESTUIN_MOCK_HW=1 — using mock sensor driver");
        return SensorDriver::Mock(MockDriver);
    }
    tracing::warn!("real sensor driver not implemented yet; falling back to mock");
    SensorDriver::Mock(MockDriver)
}

pub fn spawn_poller(
    db: Db,
    mut driver: SensorDriver,
    tx: broadcast::Sender<Reading>,
    interval: Duration,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            match tick(&db, &mut driver, &tx).await {
                Ok(()) => {}
                Err(e) => tracing::error!(?e, "sensor poll failed"),
            }
        }
    });
}

async fn tick(
    db: &Db,
    driver: &mut SensorDriver,
    tx: &broadcast::Sender<Reading>,
) -> AppResult<()> {
    let sample = driver.read().await?;
    let taken_at = Timestamp::now();
    let id = Uuid::now_v7();

    let mut db = db.clone();
    let reading = Reading {
        id,
        taken_at,
        temp_c: sample.temp_c,
        humidity: sample.humidity,
        moisture: sample.moisture,
    };

    Reading::create()
        .id(id)
        .taken_at(taken_at)
        .temp_c(reading.temp_c)
        .humidity(reading.humidity)
        .moisture(reading.moisture)
        .exec(&mut db)
        .await
        .map_err(AppError::from)?;
    let _ = tx.send(reading);
    Ok(())
}

pub fn to_centi(v: f64) -> i64 {
    (v * 100.0).round() as i64
}
