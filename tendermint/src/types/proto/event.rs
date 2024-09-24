use bytes::Bytes;
pub use tendermint_informal::abci::EventAttributeIndexExt;

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub r#type: String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: Vec<EventAttribute>,
}

impl Event {
    pub fn new(kind: &str, attr: impl IntoIterator<Item = EventAttribute>) -> Self {
        Self {
            r#type: kind.to_owned(),
            attributes: attr.into_iter().collect(),
        }
    }
}

impl From<inner::Event> for Event {
    fn from(inner::Event { r#type, attributes }: inner::Event) -> Self {
        Self {
            r#type,
            attributes: attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Event> for inner::Event {
    fn from(Event { r#type, attributes }: Event) -> Self {
        Self {
            r#type,
            attributes: attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<inner::InformalEvent> for Event {
    fn from(inner::InformalEvent { kind, attributes }: inner::InformalEvent) -> Self {
        Self {
            r#type: kind,
            attributes: attributes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Event> for inner::InformalEvent {
    type Error = std::string::FromUtf8Error;

    fn try_from(Event { r#type, attributes }: Event) -> Result<Self, Self::Error> {
        let mut attributes_res = vec![];
        for attr in attributes {
            attributes_res.push(attr.try_into()?);
        }
        Ok(Self {
            kind: r#type,
            attributes: attributes_res,
        })
    }
}

#[derive(Clone, PartialEq, Eq, ::prost::Message, serde::Serialize, serde::Deserialize)]
pub struct EventAttribute {
    #[prost(bytes = "bytes", tag = "1")]
    pub key: Bytes,
    #[prost(bytes = "bytes", tag = "2")]
    pub value: Bytes,
    /// nondeterministic
    #[prost(bool, tag = "3")]
    pub index: bool,
}

impl EventAttribute {
    pub fn new(key: Bytes, value: Bytes, index: bool) -> Self {
        Self { key, value, index }
    }
}

impl From<inner::EventAttribute> for EventAttribute {
    fn from(inner::EventAttribute { key, value, index }: inner::EventAttribute) -> Self {
        Self { key, value, index }
    }
}

impl From<EventAttribute> for inner::EventAttribute {
    fn from(EventAttribute { key, value, index }: EventAttribute) -> Self {
        Self { key, value, index }
    }
}

impl From<inner::InformalEventAttribute> for EventAttribute {
    fn from(
        inner::InformalEventAttribute { key, value, index }: inner::InformalEventAttribute,
    ) -> Self {
        Self {
            key: key.into_bytes().into(),
            value: value.into_bytes().into(),
            index,
        }
    }
}

impl TryFrom<EventAttribute> for inner::InformalEventAttribute {
    type Error = std::string::FromUtf8Error;

    fn try_from(EventAttribute { key, value, index }: EventAttribute) -> Result<Self, Self::Error> {
        Ok(Self {
            key: String::from_utf8(key.to_vec())?,
            value: String::from_utf8(value.to_vec())?,
            index,
        })
    }
}

pub(crate) mod inner {
    pub use tendermint_informal::abci::Event as InformalEvent;
    pub use tendermint_informal::abci::EventAttribute as InformalEventAttribute;
    pub use tendermint_proto::abci::Event;
    pub use tendermint_proto::abci::EventAttribute;
}
