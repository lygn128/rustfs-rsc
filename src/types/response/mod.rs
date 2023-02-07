mod list_buckets_response;
mod list_objects_response;
use std::io::Cursor;
use std::ops::IndexMut;

pub use list_buckets_response::*;
pub use list_objects_response::*;
use quick_xml::events::BytesText;
use quick_xml::Writer;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::XmlError;
use crate::utils::urlencode;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl Tag {
    pub fn new<T1: Into<String>, T2: Into<String>>(key: T1, value: T2) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct TagSet {
    #[serde(rename = "Tag", default)]
    tags: Vec<Tag>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Tagging {
    tag_set: TagSet,
}

impl Tagging {
    pub fn new() -> Self {
        Self {
            tag_set: TagSet { tags: Vec::new() },
        }
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tag_set.tags
    }

    pub fn insert<T1: Into<String>, T2: Into<String>>(&mut self, key: T1, value: T2) -> &mut Self {
        let key: String = key.into();
        let value: String = value.into();
        if !key.is_empty() && !value.is_empty() {
            let mut i = 0;
            for t in &self.tag_set.tags {
                if t.key == key {
                    break;
                }
                i = i + 1;
            }
            if i >= self.tag_set.tags.len() {
                self.tag_set.tags.push(Tag::new(key, value))
            } else {
                self.tag_set.tags.index_mut(i).value = value;
            }
        }
        self
    }

    pub fn remove<T1: Into<String>>(&mut self, key: T1) -> Option<Tag> {
        let key: String = key.into();
        let mut i = 0;
        let mut find = false;
        for t in &self.tag_set.tags {
            if t.key == key {
                find = true;
                break;
            }
            i = i + 1;
        }
        if find {
            Some(self.tag_set.tags.remove(i))
        } else {
            None
        }
    }

    pub fn to_xml(self) -> Result<Vec<u8>, XmlError> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        writer
            .create_element("Tagging")
            .write_inner_content(|writer| {
                writer.create_element("TagSet").write_inner_content(|w| {
                    for s in self.tags() {
                        w.create_element("Tag").write_inner_content(|w| {
                            w.create_element("Key")
                                .write_text_content(BytesText::new(&s.key))?;
                            w.create_element("Value")
                                .write_text_content(BytesText::new(&s.value))?;
                            Ok(())
                        })?;
                    }
                    Ok(())
                })?;
                Ok(())
            })?;
        Ok(writer.into_inner().into_inner())
    }
    pub fn to_query(&self) -> Option<String> {
        let query: String = self
            .tags()
            .iter()
            .map(|t| {
                format!(
                    "{}={}",
                    urlencode(&t.key, false),
                    urlencode(&t.value, false)
                )
            })
            .collect::<Vec<String>>()
            .join("=");
        if query.is_empty() {
            None
        } else {
            Some(query)
        }
    }
}

impl TryFrom<&str> for Tagging {
    type Error = XmlError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        quick_xml::de::from_str(&value).map_err(|x| x.into())
    }
}

impl TryInto<String> for Tagging {
    type Error = XmlError;

    fn try_into(self) -> Result<String, Self::Error> {
        quick_xml::se::to_string(&self).map_err(|x| x.into())
    }
}
