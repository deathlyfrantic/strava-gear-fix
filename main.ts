import * as data from "./data-store.ts";
import * as strava from "./strava.ts";
import log from "./logger.ts";

async function getUncheckedActivities(): Promise<strava.Activity[]> {
  let lastActivityDate = data.get("lastActivityDate");
  if (!lastActivityDate) {
    // default to getting activities for the last week
    lastActivityDate = Date.now() - (7 * 24 * 60 * 60 * 1000);
  }
  const activities = await strava.getActivitiesSince(
    new Date(lastActivityDate),
  );
  if (activities.length === 0) {
    return [];
  }
  let newest = new Date(activities[0].start_date);
  for (const activity of activities) {
    const activityDate = new Date(activity.start_date);
    if (activityDate > newest) {
      newest = activityDate;
    }
  }
  data.set("lastActivityDate", newest.valueOf());
  await data.save();
  return activities;
}

async function setBikeToTrainerForVirtualRides() {
  const trainerBikeId = data.get("trainerBikeId");
  if (!trainerBikeId) {
    throw new Error("Trainer bike id not found in data store");
  }
  log.info("Checking for new activities");
  const activities = await getUncheckedActivities();
  if (activities.length === 0) {
    log.info("No new activities found");
    return;
  }
  log.info(
    `Found ${activities.length} new activit${
      activities.length === 1 ? "y" : "ies"
    }`,
  );
  for (const activity of activities) {
    log.info(`Found new activity "${activity.name}"`);
    if (
      activity.sport_type === "VirtualRide" &&
      activity.gear_id !== trainerBikeId
    ) {
      log.info(
        `"${activity.name}" is a virtual ride, but bike is not trainer bike. Setting to trainer bike.`,
      );
      const response: strava.Activity = await strava.updateActivity(
        activity.id,
        { gear_id: trainerBikeId },
      );
      if (response.gear_id === trainerBikeId) {
        log.info(
          `Successfuly set bike to trainer bike for activity "${activity.name}"`,
        );
      } else {
        log.warning("Gear id on activity doesn't match trainer bike id", {
          activity,
          trainerBikeId,
        });
      }
    }
  }
}

if (import.meta.main) {
  setBikeToTrainerForVirtualRides();
}
