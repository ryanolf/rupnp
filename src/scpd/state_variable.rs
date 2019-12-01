use crate::find_in_xml;
use crate::Error;
use roxmltree::Node;
use std::fmt;
use std::ops::RangeInclusive;

/// A `StateVariable` is the type of every [Argument](struct.Argument.html) in UPnP Actions.
/// It is either a single value, an enumeration of strings or an integer range: see
/// [StateVariableKind](enum.StateVariableKind.html).
#[derive(Debug)]
pub struct StateVariable {
    name: String,
    default: Option<String>,
    kind: StateVariableKind,
    optional: bool,
}

/// The type of a state variable.
#[derive(Debug)]
pub enum StateVariableKind {
    /// Just a value of some datatype
    Simple(DataType),
    /// An enumeration of possible strings. Can have a default value.
    Enum(Vec<String>),
    /// A Range of integer values.
    Range(RangeInclusive<i64>, i64),
}

impl fmt::Display for StateVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl StateVariable {
    pub(crate) fn from_xml(node: Node<'_, '_>) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (name, datatype, default, variants, range, optional) = find_in_xml! { node => name, dataType, ?defaultValue, ?allowedValueList, ?allowedValueRange, ?optional };

        let variants = variants
            .map(|a| {
                a.children()
                    .filter(Node::is_element)
                    .map(crate::parse_node_text)
                    .collect()
            })
            .transpose()?;

        let default = default.map(crate::parse_node_text).transpose()?;
        let range = range.map(range_from_xml).transpose()?;

        let name = crate::parse_node_text(name)?;
        let datatype = crate::parse_node_text(datatype)?;
        let optional = optional.is_some();

        let kind = match (variants, range) {
            (None, None) => Ok(StateVariableKind::Simple(datatype)),
            (Some(variants), None) => Ok(StateVariableKind::Enum(variants)),
            (None, Some((range, step))) => Ok(StateVariableKind::Range(range, step)),
            (Some(_), Some(_)) => Err(Error::ParseError(
                "both `AllowedValues` and `AllowedValueRange` is set",
            )),
        }?;

        Ok(StateVariable {
            name,
            kind,
            default,
            optional,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_ref().map(String::as_str)
    }

    pub fn optional(&self) -> bool {
        self.optional
    }

    pub fn kind(&self) -> &StateVariableKind {
        &self.kind
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum DataType {
    ui1,
    ui2,
    ui4,
    ui8,
    i1,
    i2,
    i4,
    int,
    r4,
    r8,
    Number,
    Float,
    Fixed14_4,
    Char,
    String,
    Date,
    DateTime,
    DateTimeTz,
    Time,
    TimeTz,
    Boolean,
    BinBase64,
    BinHex,
    Uri,
}
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct ParseDataTypeErr(String);
impl fmt::Display for ParseDataTypeErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid data type: {}", self.0)
    }
}
impl std::error::Error for ParseDataTypeErr {}
impl std::str::FromStr for DataType {
    type Err = ParseDataTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ui1" => Ok(DataType::ui1),
            "ui2" => Ok(DataType::ui2),
            "ui4" => Ok(DataType::ui4),
            "ui8" => Ok(DataType::ui8),
            "i1" => Ok(DataType::i1),
            "i2" => Ok(DataType::i2),
            "i4" => Ok(DataType::i4),
            "int" => Ok(DataType::int),
            "r4" => Ok(DataType::r4),
            "r8" => Ok(DataType::r8),
            "number" => Ok(DataType::Number),
            "float" => Ok(DataType::Float),
            "fixed14_4" => Ok(DataType::Fixed14_4),
            "char" => Ok(DataType::Char),
            "string" => Ok(DataType::String),
            "date" => Ok(DataType::Date),
            "dateTime" => Ok(DataType::DateTime),
            "dateTimeTz" => Ok(DataType::DateTimeTz),
            "time" => Ok(DataType::Time),
            "timeTz" => Ok(DataType::TimeTz),
            "boolean" => Ok(DataType::Boolean),
            "bin.base64" => Ok(DataType::BinBase64),
            "bin.hex" => Ok(DataType::BinHex),
            "uri" => Ok(DataType::Uri),
            _ => Err(ParseDataTypeErr(s.to_string())),
        }
    }
}

fn range_from_xml(node: Node<'_, '_>) -> Result<(RangeInclusive<i64>, i64), Error> {
    #[allow(non_snake_case)]
    let (minimum, maximum, step) = find_in_xml! { node => minimum, maximum, ?step };

    let step = step.map(crate::parse_node_text).transpose()?.unwrap_or(1);
    let minimum = crate::parse_node_text(minimum)?;
    let maximum = crate::parse_node_text(maximum)?;

    Ok((minimum..=maximum, step))
}
