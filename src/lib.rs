
extern crate chrono;
use chrono::offset::local::Local;
use chrono::offset::TimeZone;
use chrono::datetime::DateTime;

/// The expected format for combined times and dates
const DATE_TIME_FORMAT: &'static str = "%Y/%m/%d %H:%M:%S%.f";

/// Types of messages
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    SelectionChange,
    NewId,
    NewAircraft,
    StatusAircraft,
    Click,
    /// Indicates a transmission from an aircraft. This type is most common.
    ///
    /// The type of the transmission is included.
    Transmission(TransmissionType),
}

/// Types of transmissions
#[derive(Debug, Clone, PartialEq)]
pub enum TransmissionType {
    EsIdentAndCategory,
    EsSurfacePos,
    EsAirbornePos,
    EsAirborneVel,
    SurveillanceAlt,
    SurveillanceId,
    AirToAir,
    AllCallReply,
}

/// An SBS-1 message
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// The type of the message
    pub message_type: MessageType,
    pub session_id: Option<u32>,
    pub aircraft_id: Option<u32>,
    /// Aircraft identifier
    pub ident: Option<u32>,
    pub flight_id: Option<u32>,
    /// When the message was generated
    pub generated: Option<DateTime<Local>>,
    /// When the message was logged
    pub logged: Option<DateTime<Local>>,
    /// The flight number or callsign
    pub callsign: Option<String>,
    /// The altitude of the aircraft above mean sea level, assuming an altimeter setting of
    /// 1013 millibars (29.92 inches of mercury)
    pub altitude: Option<f64>,
    /// The ground speed of the aircraft, in some unknown unit
    pub ground_speed: Option<f64>,
    /// The track of the aircraft, in degrees
    pub track: Option<f64>,
    /// The aircraft latitude
    pub latitude: Option<f64>,
    /// The aircraft longitude
    pub longitude: Option<f64>,
    /// The vertical speed of the aircraft, in some unknown unit (possibly feet per minute)
    pub vertical_speed: Option<f64>,
    /// The current transponder code
    pub squawk: Option<u16>,
    /// Indicates the transponder code has changed
    pub alert: Option<bool>,
    /// Indicates an emergency code has been set
    pub emergency: Option<bool>,
    /// Indicates the Special Position Indicator has been set
    pub special_position: Option<bool>,
    /// Indicates the aircraft is on the ground
    pub on_ground: Option<bool>,
}

impl Message {
    /// Creates a new message of the provided type, with all other fields set to None
    fn new(message_type: MessageType) -> Message {
        Message {
            message_type: message_type,
            session_id: None,
            aircraft_id: None,
            ident: None,
            flight_id: None,
            generated: None,
            logged: None,
            callsign: None,
            altitude: None,
            ground_speed: None,
            track: None,
            latitude: None,
            longitude: None,
            vertical_speed: None,
            squawk: None,
            alert: None,
            emergency: None,
            special_position: None,
            on_ground: None,
        }
    }
}

/// Errors that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// The provided message had an invalid format
    InvalidLineFormat,
    /// The message type was invalid
    InvalidMessageType,
    /// The transmission type (for messages of type Transmission) was invalid
    InvalidTransmissionType,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Parse error: {}", ::std::error::Error::description(self))
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidLineFormat => "Invalid line format",
            ParseError::InvalidMessageType => "Invalid message type",
            ParseError::InvalidTransmissionType => "Invalid transmission type",
        }
    }
}

/// Parses a line of text into a message
pub fn parse(message_string: &str) -> Result<Message, ParseError> {
    let parts = message_string.trim().split(',').collect::<Vec<_>>();
    if parts.len() != 22 {
        return Err(ParseError::InvalidLineFormat);
    }
    let message_type = match parts[0] {
        "SEL" => MessageType::SelectionChange,
        "ID" => MessageType::NewId,
        "AIR" => MessageType::NewAircraft,
        "STA" => MessageType::StatusAircraft,
        "CLK" => MessageType::Click,
        "MSG" => {
            // Transmission message
            let transmission_type = match parts[1] {
                "1" => TransmissionType::EsIdentAndCategory,
                "2" => TransmissionType::EsSurfacePos,
                "3" => TransmissionType::EsAirbornePos,
                "4" => TransmissionType::EsAirborneVel,
                "5" => TransmissionType::SurveillanceAlt,
                "6" => TransmissionType::SurveillanceId,
                "7" => TransmissionType::AirToAir,
                "8" => TransmissionType::AllCallReply,
                _ => return Err(ParseError::InvalidTransmissionType),
            };
            MessageType::Transmission(transmission_type)
        }
        _ => return Err(ParseError::InvalidMessageType),
    };
    // Create a message
    let mut message = Message::new(message_type);
    // Fill in fields
    message.session_id = parts[2].parse().ok();
    message.aircraft_id = parts[3].parse().ok();
    message.ident = parts[4].parse().ok();
    message.flight_id = parts[5].parse().ok();
    message.generated = parse_date_time(parts[6], parts[7]).ok();
    message.logged = parse_date_time(parts[8], parts[9]).ok();
    message.callsign = if parts[10].is_empty() { None } else { Some(String::from(parts[10].trim())) };
    message.altitude = parts[11].parse().ok();
    message.ground_speed = parts[12].parse().ok();
    message.track = parts[13].parse().ok();
    message.latitude = parts[14].parse().ok();
    message.longitude = parts[15].parse().ok();
    message.vertical_speed = parts[16].parse().ok();
    message.squawk = parts[17].parse().ok();
    message.alert = parts[18].parse().ok();
    message.emergency = parts[19].parse().ok();
    message.special_position = parts[20].parse().ok();
    message.on_ground = parts[21].parse().ok();

    Ok(message)
}

/// Parses a date component and a time component into a DateTime
fn parse_date_time(date: &str, time: &str) -> Result<DateTime<Local>, chrono::format::ParseError> {
    let combined = format!("{} {}", date.trim(), time.trim());
    Local.datetime_from_str(&combined, DATE_TIME_FORMAT)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_string() {
        let result = parse("");
        assert_eq!(Err(ParseError::InvalidLineFormat), result);
    }
    #[test]
    fn test_few_commas() {
        let result = parse(",,,,");
        assert_eq!(Err(ParseError::InvalidLineFormat), result);
    }
    #[test]
    fn test_correct_commas_empty() {
        let result = parse(",,,,,,,,,,,,,,,,,,,,,");
        assert_eq!(Err(ParseError::InvalidMessageType), result);
    }
    #[test]
    fn test_selection_change() {
        let result = parse("SEL,,,,,,,,,,,,,,,,,,,,,");
        let expected = Ok(Message::new(MessageType::SelectionChange));
        assert_eq!(expected, result);
    }
    #[test]
    fn test_selection_change_vertical_speed() {
        let result = parse("SEL,,,,,,,,,,,,,,,,-350,,,,,");
        assert!(!result.is_err());
        let result = result.unwrap();
        assert_eq!(MessageType::SelectionChange, result.message_type);
        assert!(result.vertical_speed.is_some());
        assert_eq!(-350_f64, result.vertical_speed.unwrap());
    }

    #[test]
    fn test_date_time_parse() {
        let date = "2016/03/11";
        let time = "21:24:53.351";
        let result = super::parse_date_time(date, time);
        match result {
            Ok(_) => {},
            Err(e) => {
                println!("{:?}", e);
                assert!(false);
            },
        }
    }
}
