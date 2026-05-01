use boa_engine::{property::PropertyKey, JsString, JsSymbol};

/// A compatibility-focused subset of QuickJS predefined atoms for the Boa backend.
///
/// This currently covers the property keys exercised by the macro/class surface and nearby
/// helper APIs. Symbol entries are mapped to Boa well-known symbols through native Rust APIs.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum PredefinedAtom {
    Name,
    Length,
    Prototype,
    Constructor,
    Message,
    Stack,
    Then,
    Catch,
    Value,
    Done,
    Next,
    ToJSON,
    ToString,
    SymbolAsyncIterator,
    SymbolIterator,
    SymbolMatch,
    SymbolMatchAll,
    SymbolReplace,
    SymbolSearch,
    SymbolSplit,
    SymbolToPrimitive,
    SymbolToStringTag,
    SymbolIsConcatSpreadable,
    SymbolHasInstance,
    SymbolSpecies,
    SymbolUnscopables,
}

impl PredefinedAtom {
    pub const fn is_symbol(self) -> bool {
        matches!(
            self,
            Self::SymbolAsyncIterator
                | Self::SymbolIterator
                | Self::SymbolMatch
                | Self::SymbolMatchAll
                | Self::SymbolReplace
                | Self::SymbolSearch
                | Self::SymbolSplit
                | Self::SymbolToPrimitive
                | Self::SymbolToStringTag
                | Self::SymbolIsConcatSpreadable
                | Self::SymbolHasInstance
                | Self::SymbolSpecies
                | Self::SymbolUnscopables
        )
    }

    pub const fn to_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Length => "length",
            Self::Prototype => "prototype",
            Self::Constructor => "constructor",
            Self::Message => "message",
            Self::Stack => "stack",
            Self::Then => "then",
            Self::Catch => "catch",
            Self::Value => "value",
            Self::Done => "done",
            Self::Next => "next",
            Self::ToJSON => "toJSON",
            Self::ToString => "toString",
            Self::SymbolAsyncIterator => "Symbol.asyncIterator",
            Self::SymbolIterator => "Symbol.iterator",
            Self::SymbolMatch => "Symbol.match",
            Self::SymbolMatchAll => "Symbol.matchAll",
            Self::SymbolReplace => "Symbol.replace",
            Self::SymbolSearch => "Symbol.search",
            Self::SymbolSplit => "Symbol.split",
            Self::SymbolToPrimitive => "Symbol.toPrimitive",
            Self::SymbolToStringTag => "Symbol.toStringTag",
            Self::SymbolIsConcatSpreadable => "Symbol.isConcatSpreadable",
            Self::SymbolHasInstance => "Symbol.hasInstance",
            Self::SymbolSpecies => "Symbol.species",
            Self::SymbolUnscopables => "Symbol.unscopables",
        }
    }

    pub(crate) fn to_property_key(self) -> PropertyKey {
        match self {
            Self::SymbolAsyncIterator => JsSymbol::async_iterator().into(),
            Self::SymbolIterator => JsSymbol::iterator().into(),
            Self::SymbolMatch => JsSymbol::r#match().into(),
            Self::SymbolMatchAll => JsSymbol::match_all().into(),
            Self::SymbolReplace => JsSymbol::replace().into(),
            Self::SymbolSearch => JsSymbol::search().into(),
            Self::SymbolSplit => JsSymbol::split().into(),
            Self::SymbolToPrimitive => JsSymbol::to_primitive().into(),
            Self::SymbolToStringTag => JsSymbol::to_string_tag().into(),
            Self::SymbolIsConcatSpreadable => JsSymbol::is_concat_spreadable().into(),
            Self::SymbolHasInstance => JsSymbol::has_instance().into(),
            Self::SymbolSpecies => JsSymbol::species().into(),
            Self::SymbolUnscopables => JsSymbol::unscopables().into(),
            _ => PropertyKey::from(JsString::from(self.to_str())),
        }
    }
}
