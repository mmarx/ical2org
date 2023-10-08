use std::{
    collections::HashSet,
    io::{BufRead, BufWriter, Write},
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Local};
use ical::{
    parser::ical::component::{IcalCalendar, IcalEvent},
    property::Property,
    IcalParser,
};
use rrule::{RRuleSet, Tz};

use crate::datetime::{is_midnight, org_date, org_datetime};

const ONCE: &str = "RRULE:FREQ=DAILY;COUNT=1";
const NO_TITLE: &str = "(No title)";
const RECUR_TAG: &str = ":RECURRING:";
const RR_PROPERTIES: [&str; 5] = ["RRULE", "EXRULE", "DTSTART", "EXDATE", "RDATE"];

/// The main event converter
pub struct Converter {
    /// Days to the left and right of the current day.
    days: u32,
    /// List of user emails (if any of these declined, the event is ignored).
    emails: HashSet<String>,
    /// Whether to include the location in converted events.
    include_location: bool,
    /// Whether to continue on errors.
    continue_on_error: bool,
    tz: Tz,
    rr_props: HashSet<String>,
}

impl Converter {
    pub fn new(
        days: u32,
        emails: Vec<String>,
        tz: Option<chrono_tz::Tz>,
        include_location: bool,
        continue_on_error: bool,
    ) -> Self {
        let mut rr_props = HashSet::new();

        for prop in RR_PROPERTIES {
            rr_props.insert(prop.to_string());
        }

        Self {
            days,
            emails: emails.into_iter().collect(),
            tz: tz.map(|tz| tz.into()).unwrap_or(Tz::LOCAL),
            include_location,
            continue_on_error,
            rr_props,
        }
    }

    pub fn convert(&self, ics_file: impl BufRead, org_file: &mut impl Write) -> Result<()> {
        let reader = IcalParser::new(ics_file);
        let window = Duration::days(self.days.into());

        let now = Local::now().with_timezone(&self.tz);
        let start = now - window;
        let end = now + window;

        log::debug!("Using window: {start:?}--{end:?}");

        for calendar in reader {
            match calendar {
                Ok(calendar) => match self.convert_calendar(calendar, &mut *org_file, &start, &end)
                {
                    Ok(()) => (),
                    Err(err) => {
                        log::error!("Failed to convert calendar: {err:?}");
                        if !self.continue_on_error {
                            return Err(err);
                        }
                    }
                },
                Err(err) => {
                    log::error!("Failed to parse calendar: {err:?}");
                    if !self.continue_on_error {
                        return Err(err.into());
                    }
                }
            }
        }

        org_file.flush()?;

        Ok(())
    }

    fn convert_calendar(
        &self,
        calendar: IcalCalendar,
        org_file: &mut impl Write,
        start: &DateTime<Tz>,
        end: &DateTime<Tz>,
    ) -> Result<()> {
        let mut result = Vec::new();
        for event in calendar.events {
            match self.convert_event(event, &mut BufWriter::new(&mut result), start, end) {
                Ok(()) => (),
                Err(err) => {
                    log::error!("Failed to convert event: {err:?}");
                    if !self.continue_on_error {
                        return Err(err);
                    }
                }
            }
        }

        org_file.write_all(&result)?;
        org_file.flush()?;

        Ok(())
    }

    fn convert_event(
        &self,
        event: IcalEvent,
        org_file: &mut impl Write,
        start: &DateTime<Tz>,
        end: &DateTime<Tz>,
    ) -> Result<()> {
        let mut result = Vec::new();
        let mut rrule = Vec::new();
        let mut recurs = false;
        let mut dtstart = String::new();

        for property in event.properties.iter() {
            if property.name == "RRULE" || property.name == "RDATE" {
                recurs = true;
            } else if property.name == "DTSTART" {
                dtstart = property.value.clone().expect("event should have a DTSTART");
            }

            if self.rr_props.contains(&property.name) {
                rrule.push(Self::format_property(property))
            }

            if property.name == "ATTENDEE" && self.has_declined(property) {
                log::debug!("ignoring declined event");
                return Ok(());
            }
        }

        if !recurs {
            rrule.push(ONCE.to_string());
        }

        let rrule_string = rrule.join("\n");

        log::debug!("dtstart: {dtstart:?}");
        log::debug!("constructed rrule: {rrule_string:?}");

        match rrule_string.parse::<RRuleSet>() {
            Ok(rrule_set) => {
                let instances = rrule_set.after(*start).before(*end).all(65535);

                for instance in instances.dates {
                    match self.convert_event_instance(
                        &event,
                        &mut BufWriter::new(&mut result),
                        &instance,
                    ) {
                        Ok(()) => (),
                        Err(err) => {
                            log::error!("Failed to convert event instance: {err:?}");
                            if !self.continue_on_error {
                                return Err(err);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to parse RRuleSet: {err:?}");
                return Err(err.into());
            }
        }

        org_file.write_all(&result)?;
        org_file.flush()?;

        Ok(())
    }

    fn convert_event_instance(
        &self,
        event: &IcalEvent,
        org_file: &mut impl Write,
        date: &DateTime<Tz>,
    ) -> Result<()> {
        let mut summary = None;
        let mut location = None;
        let mut description = None;
        let mut dtstart = None;
        let mut dtend = None;
        let mut duration = None;
        let mut recurs = false;
        let mut whole_day = false;

        for property in event.properties.iter() {
            match property.name.as_str() {
                "SUMMARY" => summary = property.value.clone(),
                "LOCATION" => location = property.value.clone(),
                "DESCRIPTION" => description = property.value.clone(),
                "DTSTART" => dtstart = Some(Self::datetime_from_property(property)?),
                "DTEND" => dtend = Some(Self::datetime_from_property(property)?),
                "DURATION" => {
                    duration =
                        iso8601::duration(&property.value.clone().expect("should not be empty"))
                            .map_err(|err| anyhow!(err))?
                            .into()
                }
                "RRULE" => recurs = true,
                _ => (),
            }
        }

        if let Some(duration) = duration {
            if let Some(start) = dtstart {
                dtend = Some(start + Duration::from_std(duration.into())?)
            }
        }

        let title = match summary {
            None => match location {
                None => NO_TITLE.to_string(),
                Some(location) => location.clone(),
            },
            Some(summary) => match location {
                Some(location) if self.include_location => format!("{summary} - {location}"),
                _ => summary.clone(),
            },
        };

        // heading

        write!(org_file, "* {title}")?;
        if recurs {
            write!(org_file, " {RECUR_TAG}")?;
        }
        writeln!(org_file)?;

        // timestamps

        if let (Some(mut start), Some(mut end)) = (dtstart, dtend) {
            let duration = end - start;

            if recurs {
                start = *date;
                end = start + duration;
            }

            if is_midnight(start.time()) && is_midnight(end.time()) {
                // whole-day event
                whole_day = true;
            }

            if is_midnight(end.time()) && duration == Duration::days(1) {
                // single-day event
                writeln!(
                    org_file,
                    "  {}",
                    if whole_day {
                        org_date(start, &self.tz)
                    } else {
                        org_datetime(start, &self.tz)
                    }
                )?;
            } else {
                writeln!(
                    org_file,
                    "  {}--{}",
                    if whole_day {
                        org_date(start, &self.tz)
                    } else {
                        org_datetime(start, &self.tz)
                    },
                    if whole_day {
                        org_date(end, &self.tz)
                    } else {
                        org_datetime(end, &self.tz)
                    }
                )?;
            }
        }

        // description

        if let Some(description) = description {
            writeln!(org_file, "{description}")?;
        }

        writeln!(org_file)?;
        org_file.flush()?;

        Ok(())
    }

    fn format_property(property: &Property) -> String {
        let mut parameters = Vec::new();

        match &property.params {
            None => (),
            Some(params) => {
                for (param, values) in params {
                    parameters.push(format!("{param}={}", values.join(",")));
                }
            }
        }

        format!(
            "{}{}{}:{}",
            property.name,
            if parameters.is_empty() { "" } else { ";" },
            parameters.join(";"),
            property.value.clone().unwrap_or_default()
        )
    }

    fn datetime_from_property(property: &Property) -> Result<DateTime<Tz>> {
        let mut prop = property.clone();
        prop.name = "DTSTART".to_string();

        Ok(*[ONCE.to_string(), Self::format_property(&prop)]
            .join("\n")
            .parse::<RRuleSet>()?
            .all(1)
            .dates
            .first()
            .expect("should not be empty"))
    }

    fn has_declined(&self, property: &Property) -> bool {
        let mut declined = false;
        let mut is_self = false;

        if let Some(parameters) = &property.params {
            for (parameter, values) in parameters {
                if parameter == "CN" {
                    for value in values {
                        if self.emails.contains(value) {
                            is_self = true;
                        }
                    }
                } else if parameter == "PARTSTAT" && values.contains(&"DECLINED".to_string()) {
                    declined = true;
                }
            }
        }

        declined && is_self
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new(90, Vec::new(), None, true, false)
    }
}
