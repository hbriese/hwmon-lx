// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use lazy_static::lazy_static;

use crate::error::*;
use crate::feature::FeatureType;
use crate::prefix::si::*;
use crate::ratio::Ratio;
use crate::sysfs::*;

macro_rules! make_subfeatures {
    (feature: $Feature:ident, map: $MAP_NAME:ident, variants: [ $($Variant:ident { $pattern:expr, $ratio:ident, $alarm:expr}),* $(,)* ]) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
        pub enum $Feature {
            $($Variant),*
        }

        impl $Feature {
            fn ratio(self) -> &'static Ratio<u64> {
                match self {
                    $($Feature::$Variant => &$ratio,)*
                }
            }

            /// Return `true` if the subfeature variant is an alarm.
            pub fn is_alarm(self) -> bool {
                match self {
                    $($Feature::$Variant => $alarm,)*
                }
            }
        }

        lazy_static! {
            static ref $MAP_NAME: HashMap<&'static str, SubfeatureType> = {
                let mut m = HashMap::new();
                $(m.insert($pattern, SubfeatureType::$Feature($Feature::$Variant));)*
                m.shrink_to_fit();
                m
            };
        }
    }
}

make_subfeatures! {
    feature: Fan,
    map: FAN_MAP,
    variants: [
        Input { "input", Unity, false },
        Min { "min", Unity, false },
        Max { "max", Unity, false },
        Div { "div", Unity, false },
        Pulses { "pulses", Unity, false },
        Target { "target", Unity, false },
        Label { "label", Unity, false },
        Enable { "enable", Unity, false },
        // Alarms
        Alarm { "alarm", Unity, true },
        Min_Alarm { "min_alarm", Unity, true },
        Max_Alarm { "max_alarm", Unity, true },
        Fault { "fault", Unity, false },
        Beep { "beep", Unity, false },
    ]
}

make_subfeatures! {
    feature: Pwm,
    map: PWM_MAP,
    variants: [
        Pwm { "", Unity, false },
        Enable { "enable", Unity, false },
        Mode { "mode", Unity, false },
        Freq { "freq", Unity, false },
    ]
}

make_subfeatures! {
    feature: Temperature,
    map: TEMPERATURE_MAP,
    variants: [
        Input { "input", Milli, false },
        Max { "max", Milli, false },
        Max_Hyst { "max_hyst", Milli, false },
        Min { "min", Milli, false },
        Min_Hyst { "min_hyst", Milli, false },
        Crit_Max { "crit", Milli, false },
        Crit_Max_Hyst { "crit_hyst", Milli, false },
        Crit_Min { "lcrit", Milli, false },
        Crit_Min_Hyst { "lcrit_hyst", Milli, false },
        Emergency { "emergency", Milli, false },
        Emergency_Hyst { "emergency_hyst", Milli, false },
        Lowest { "lowest", Milli, false },
        Highest { "highest", Milli, false },
        Offset { "offset", Milli, false },
        Type { "type", Unity, false },
        // Alarms
        Alarm { "alarm", Unity, true },
        Max_Alarm { "max_alarm", Unity, true },
        Min_Alarm { "min_alarm", Unity, true },
        Emergency_Alarm { "emergency_alarm", Unity, true },
        Crit_Max_Alarm { "crit_alarm", Unity, true },
        Crit_Min_Alarm { "lcrit_alarm", Unity, true },
        Fault { "fault", Unity, false },
        Beep { "beep", Unity, false },
    ]
}

make_subfeatures! {
    feature: Voltage,
    map: VOLTAGE_MAP,
    variants: [
        Input { "input", Milli, false },
        Max { "max", Milli, false },
        Min { "min", Milli, false },
        Crit_Max { "crit", Milli, false },
        Crit_Min { "lcrit", Milli, false },
        Average { "average", Milli, false },
        Highest { "highest", Milli, false },
        Lowest { "lowest", Milli, false },
        // Alarms
        Alarm { "alarm", Unity, true },
        Max_Alarm { "max_alarm", Unity, true },
        Min_Alarm { "min_alarm", Unity, true },
        Crit_Max_Alarm { "crit_alarm", Unity, true },
        Crit_Min_Alarm { "lcrit_alarm", Unity, true },
        Beep { "beep", Unity, false },
    ]
}

make_subfeatures! {
    feature: Current,
    map: CURRENT_MAP,
    variants: [
        Input { "input", Milli, false },
        Max { "max", Milli, false },
        Min { "min", Milli, false },
        Crit_Max { "crit", Milli, false },
        Crit_Min { "lcrit", Milli, false },
        Average { "average", Milli, false },
        Highest { "highest", Milli, false },
        Lowest { "lowest", Milli, false },
        // Alarms
        Alarm { "alarm", Unity, true },
        Max_Alarm { "max_alarm", Unity, true },
        Min_Alarm { "min_alarm", Unity, true },
        Crit_Max_Alarm { "crit_alarm", Unity, true },
        Crit_Min_Alarm { "lcrit_alarm", Unity, true },
        Beep { "beep", Unity, false },
    ]
}

make_subfeatures! {
    feature: Power,
    map: POWER_MAP,
    variants: [
        Average { "average", Micro, false },
        Average_Highest { "average_highest", Micro, false },
        Average_Lowest { "average_lowest", Micro, false },
        Input { "input", Micro, false },
        Input_Highest { "input_highest", Micro, false },
        Input_Lowest { "input_lowest", Micro, false },
        Cap { "cap", Micro, false },
        Cap_Max { "cap_max", Micro, false },
        Cap_Min { "cap_min", Micro, false },
        Cap_Hyst { "cap_hyst", Micro, false },
        Max { "max", Micro, false },
        Min { "min", Micro, false },
        Crit_Max { "crit", Micro, false },
        Crit_Min { "lcrit", Micro, false },
        Average_Interval { "average_interval", Milli, false },
        Average_Interval_Max { "average_interval_max", Milli, false },
        Average_Interval_Min { "average_interval_min", Milli, false },
        Accuracy { "accuracy", Unity, false },
        // Alarms
        Alarm { "alarm", Unity, true },
        Cap_Alarm { "cap_alarm", Unity, true },
        Max_Alarm { "max_alarm", Unity, true },
        Min_Alarm { "min_alarm", Unity, true },
        Crit_Max_Alarm { "crit_alarm", Unity, true },
        Crit_Min_Alarm { "lcrit_alarm", Unity, true },
    ]
}

make_subfeatures! {
    feature: Energy,
    map: ENERGY_MAP,
    variants: [
        Input { "input", Micro, false },
    ]
}

make_subfeatures! {
    feature: Humidity,
    map: HUMIDITY_MAP,
    variants: [
        Input { "input", Milli, false },
    ]
}

make_subfeatures! {
    feature: Intrusion,
    map: INTRUSION_MAP,
    variants: [
        Alarm { "alarm", Micro, false },
        Beep { "beep", Micro, false },
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubfeatureType {
    Fan(Fan),
    Pwm(Pwm),
    Temperature(Temperature),
    Voltage(Voltage),
    Current(Current),
    Power(Power),
    Energy(Energy),
    Humidity(Humidity),
    Cpu,
    Intrusion(Intrusion),
    BeepEnable,
}

impl SubfeatureType {
    fn to_native(self, value: f64) -> i64 {
        (value * *self.ratio().denom() as f64 / *self.ratio().numer() as f64).round() as i64
    }

    fn to_unity(self, value: f64) -> f64 {
        value * *self.ratio().numer() as f64 / *self.ratio().denom() as f64
    }

    fn ratio(self) -> &'static Ratio<u64> {
        match self {
            SubfeatureType::Fan(sft) => sft.ratio(),
            SubfeatureType::Pwm(sft) => sft.ratio(),
            SubfeatureType::Temperature(sft) => sft.ratio(),
            SubfeatureType::Voltage(sft) => sft.ratio(),
            SubfeatureType::Current(sft) => sft.ratio(),
            SubfeatureType::Power(sft) => sft.ratio(),
            SubfeatureType::Energy(sft) => sft.ratio(),
            SubfeatureType::Humidity(sft) => sft.ratio(),
            SubfeatureType::Intrusion(sft) => sft.ratio(),
            SubfeatureType::Cpu => &Milli,
            SubfeatureType::BeepEnable => &Unity,
        }
    }

    /// Return `true` if the subfeature variant is an alarm.
    pub fn is_alarm(self) -> bool {
        match self {
            SubfeatureType::Fan(sft) => sft.is_alarm(),
            SubfeatureType::Pwm(sft) => sft.is_alarm(),
            SubfeatureType::Temperature(sft) => sft.is_alarm(),
            SubfeatureType::Voltage(sft) => sft.is_alarm(),
            SubfeatureType::Current(sft) => sft.is_alarm(),
            SubfeatureType::Power(sft) => sft.is_alarm(),
            SubfeatureType::Energy(sft) => sft.is_alarm(),
            SubfeatureType::Humidity(sft) => sft.is_alarm(),
            SubfeatureType::Intrusion(sft) => sft.is_alarm(),
            SubfeatureType::Cpu => false,
            SubfeatureType::BeepEnable => false,
        }
    }
}

lazy_static! {
    static ref CPU_MAP: HashMap<&'static str, SubfeatureType> = {
        use self::SubfeatureType::*;

        let mut m = HashMap::new();
        m.insert("vid", Cpu);
        m.shrink_to_fit();
        m
    };
    static ref FEATURE_TYPE_MAP: HashMap<&'static str, (FeatureType, &'static HashMap<&'static str, SubfeatureType>)> = {
        let mut m: HashMap<
            &'static str,
            (FeatureType, &'static HashMap<&'static str, SubfeatureType>),
        > = HashMap::new();
        m.insert("temp", (FeatureType::Temperature, &TEMPERATURE_MAP));
        m.insert("in", (FeatureType::Voltage, &VOLTAGE_MAP));
        m.insert("fan", (FeatureType::Fan, &FAN_MAP));
        m.insert("pwm", (FeatureType::Pwm, &PWM_MAP));
        m.insert("cpu", (FeatureType::Cpu, &CPU_MAP));
        m.insert("power", (FeatureType::Power, &POWER_MAP));
        m.insert("curr", (FeatureType::Current, &CURRENT_MAP));
        m.insert("energy", (FeatureType::Energy, &ENERGY_MAP));
        m.insert("intrusion", (FeatureType::Intrusion, &INTRUSION_MAP));
        m.insert("humidity", (FeatureType::Humidity, &HUMIDITY_MAP));
        m.shrink_to_fit();
        m
    };
}

#[derive(Clone, Debug)]
pub struct Subfeature {
    name: String,
    path: PathBuf,
    subfeature_type: SubfeatureType,
    compute_statement: Option<String>,
    is_readable: bool,
    is_writable: bool,
}

impl Subfeature {
    /// Subfeature name
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Return the sysfs file path
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Get the subfeature type
    pub fn get_type(&self) -> SubfeatureType {
        self.subfeature_type
    }

    /// Return the compute statement string if specified in the configuration file.
    /// Otherwise it return None.
    pub fn compute_statement(&self) -> Option<String> {
        self.compute_statement.clone()
    }

    /// Return `true` if the subfeature is readable
    pub fn is_readable(&self) -> bool {
        self.is_readable
    }

    /// Return `true` if the subfeature is writable
    pub fn is_writable(&self) -> bool {
        self.is_writable
    }

    /// Read the value of the subfeature.
    pub fn read_value(&self) -> Result<f64, Error> {
        if self.is_readable() {
            // TODO compute statement
            self.read_sysfs_value()
        } else {
            Err(Error::Access("Subfeature not readable"))
        }
    }

    /// Write the value of the subfeature.
    ///
    /// ## Warning:
    ///
    /// No checks are made on the value before writing it.
    /// Affect a new value at your own risk.
    /// See hwmon and device driver documentation for more information.
    pub fn write_value(&self, value: f64) -> Result<(), Error> {
        if self.is_writable() {
            // TODO compute statement
            self.write_sysfs_value(value)?;
            Ok(())
        } else {
            Err(Error::Access("Subfeature not writable"))
        }
    }

    /// Read the value from sysfs file and apply the proper type scaling.
    ///
    /// Note: This function does not take into account the configuration file.
    fn read_sysfs_value(&self) -> Result<f64, Error> {
        let value = sysfs_read_file(&self.path)?.parse::<f64>()?;
        Ok(self.subfeature_type.to_unity(value))
    }

    /// Write the value to sysfs file. Before it apply the proper type scaling.
    ///
    /// Note: This function does not take into account the configuration file.
    fn write_sysfs_value(&self, value: f64) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .open(&self.path)?;
        write!(file, "{}", self.subfeature_type.to_native(value))
    }

    pub(crate) fn from_path<P: AsRef<Path>>(path: P) -> Result<(u32, Subfeature), SubfeatureError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(SubfeatureError::Invalid);
        }

        let name = path.file_name().and_then(OsStr::to_str).unwrap();

        let (feature_number, subfeature_type) = Subfeature::get_properties_from_name(name)?;

        let st_mode = path.metadata().map(|m| m.st_mode())?;
        let is_readable = (st_mode & libc::S_IRUSR) == libc::S_IRUSR;
        let is_writable = (st_mode & libc::S_IWUSR) == libc::S_IWUSR;

        Ok((
            feature_number,
            Subfeature {
                name: name.to_string(),
                path: path.to_path_buf(),
                subfeature_type,
                compute_statement: None, // TODO compute statement
                is_readable,
                is_writable,
            },
        ))
    }

    fn get_properties_from_name(name: &str) -> Result<(u32, SubfeatureType), SubfeatureError> {
        if name == "beep_enable" {
            return Ok((0, SubfeatureType::BeepEnable));
        }

        // Subfeature name Regex: "^([[:^digit:]]+)([[:digit:]]+)(_(.+))?$"
        let (subfeature_name, subfeature_id) = (name.split_once('_')).unwrap_or((name, ""));

        let feature_id_len = subfeature_name
            .find(|c: char| c.is_ascii_digit())
            .ok_or(SubfeatureError::Invalid)?;
        let (feature_id, feature_number_str) = subfeature_name.split_at(feature_id_len);
        let feature_number = u32::from_str(feature_number_str)?;

        if let Some(sf_type) = FEATURE_TYPE_MAP
            .get(feature_id)
            .and_then(|(_, sf_map)| sf_map.get(subfeature_id))
        {
            Ok((feature_number, *sf_type))
        } else {
            Err(SubfeatureError::Unknown)
        }
    }
}
