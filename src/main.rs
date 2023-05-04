use chrono::{Duration, Utc};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{collections::HashMap, io::Result};
use strava_gear_fix::{
    data_store::DataStore,
    strava::{self, Activity},
};

async fn get_unchecked_activities(data: &mut DataStore) -> Result<Vec<Activity>> {
    let last_activity_date = data
        .last_activity_date
        .unwrap_or_else(|| Utc::now() - Duration::weeks(1));
    let activities = strava::get_activities_since(last_activity_date, data).await?;
    if activities.is_empty() {
        return Ok([].into());
    }
    let newest = activities.iter().map(|activity| activity.start_date).max();
    data.last_activity_date = newest;
    data.save()?;
    Ok(activities)
}

async fn set_bike_to_trainer_for_virtual_rides(data: &mut DataStore) -> Result<()> {
    log::info!("Checking for new activities");
    let activities = get_unchecked_activities(data).await?;
    if activities.is_empty() {
        log::info!("No new activities found.");
        return Ok(());
    }
    log::info!(
        "Found {} new activit{}",
        activities.len(),
        if activities.len() == 1 { "y" } else { "ies" }
    );
    for activity in activities {
        log::info!("Found new activity \"{}\"", activity.name);
        if activity.sport_type == "VirtualRide" && activity.gear_id != data.trainer_bike_id {
            log::info!(
                "\"{}\" is a virtual ride, but bike is not trainer bike. Setting to trainer bike.",
                activity.name
            );
            let response = strava::update_activity(
                activity.id,
                HashMap::from([("gear_id".into(), data.trainer_bike_id.clone())]),
                data,
            )
            .await?;
            if response.gear_id == data.trainer_bike_id {
                log::info!(
                    "Successfuly set bike to trainer bike for activity \"{}\"",
                    activity.name
                );
            } else {
                log::warn!(
                    "Gear id on activity \"{}\" doesn't match trainer bike id \"{}\"",
                    activity.id,
                    data.trainer_bike_id
                );
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .init()
        .expect("Failed to initiate logger");
    let mut data = DataStore::load().expect("failed to load data");
    set_bike_to_trainer_for_virtual_rides(&mut data).await
}
