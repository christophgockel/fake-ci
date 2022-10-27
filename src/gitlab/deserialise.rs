use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

pub fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

//
// The `needs`keyword can contain different shapes:
//
//     needs:
//       - job: build_job1
//         artifacts: true
//       - job: build_job2
//       - build_job3
//
// A custom deserializer is used to bring both map and string versions into one single struct.
// Original implementation from: https://github.com/serde-rs/serde/issues/901
//
pub fn seq_string_or_struct<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = String>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = String>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            FromStr::from_str(value).map_err(de::Error::custom)
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    // This is a common trick that enables passing a Visitor to the
    // `seq.next_element` call below.
    impl<'de, T> DeserializeSeed<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = String>,
    {
        type Value = T;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }
    }

    struct SeqStringOrStruct<T>(PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for SeqStringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = String>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("sequence of strings or maps")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            // Tell it which Visitor to use by passing one in.
            while let Some(element) = seq.next_element_seed(StringOrStruct(PhantomData))? {
                vec.push(element);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_seq(SeqStringOrStruct(PhantomData))
}

pub fn string_hashmap<'de, D>(deserializer: D) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<String, String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("map")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut values = HashMap::new();

            while let Some((key, value)) = access.next_entry::<String, Value>()? {
                let value = match value {
                    Value::Null => "null".into(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::String(s) => s,
                    _ => return Err(A::Error::custom("Can only put primitive types into map")),
                };

                values.insert(key, value);
            }

            Ok(values)
        }
    }

    let visitor = MapVisitor;
    deserializer.deserialize_map(visitor)
}
