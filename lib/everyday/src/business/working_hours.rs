use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeeklyWorkingHours {
    pub mon: DayWorkingHours,
    pub tue: DayWorkingHours,
    pub wed: DayWorkingHours,
    pub thu: DayWorkingHours,
    pub fri: DayWorkingHours,
    pub sat: DayWorkingHours,
    pub sun: DayWorkingHours,
}

impl WeeklyWorkingHours {
    pub fn new(
        mon: DayWorkingHours,
        tue: DayWorkingHours,
        wed: DayWorkingHours,
        thu: DayWorkingHours,
        fri: DayWorkingHours,
        sat: DayWorkingHours,
        sun: DayWorkingHours,
    ) -> Self {
        Self {
            mon,
            tue,
            wed,
            thu,
            fri,
            sat,
            sun,
        }
    }
}

impl WeeklyWorkingHours {
    pub fn closed() -> Self {
        let closed_day = DayWorkingHours { intervals: vec![] };
        Self {
            mon: closed_day.clone(),
            tue: closed_day.clone(),
            wed: closed_day.clone(),
            thu: closed_day.clone(),
            fri: closed_day.clone(),
            sat: closed_day.clone(),
            sun: closed_day,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayWorkingHours {
    pub intervals: Vec<WorkingHours>,
}

impl DayWorkingHours {
    pub fn new(intervals: Vec<WorkingHours>) -> Self {
        Self { intervals }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkingHours {
    pub start: WorkingTime,
    pub end: WorkingTime,
}

impl WorkingHours {
    pub fn new(start: WorkingTime, end: WorkingTime) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkingTime {
    pub hour: u32,
    pub minute: u32,
}
impl WorkingTime {
    pub fn new(hour: u32, minute: u32) -> Self {
        Self { hour, minute }
    }
}
