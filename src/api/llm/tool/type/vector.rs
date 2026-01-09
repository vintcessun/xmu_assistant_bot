use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
pub struct LlmVec<T>(pub Vec<T>);

impl<T> Deref for LlmVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for LlmVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Vec<T>> for LlmVec<T> {
    fn from(v: Vec<T>) -> Self {
        LlmVec(v)
    }
}

impl<T> From<LlmVec<T>> for Vec<T> {
    fn from(lv: LlmVec<T>) -> Self {
        lv.0
    }
}

impl<'a, T> IntoIterator for &'a LlmVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> IntoIterator for LlmVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'de, T> Deserialize<'de> for LlmVec<T>
where
    T: Deserialize<'de> + FromStr + Debug,
    <T as FromStr>::Err: Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct XmlSeq<T> {
            #[serde(rename = "item")]
            items: Vec<T>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper<T> {
            Full(XmlSeq<T>),
            Empty(serde::de::IgnoredAny),
        }

        let result = Helper::<T>::deserialize(deserializer);

        match result {
            Ok(Helper::Full(wrapper)) => Ok(LlmVec(wrapper.items)),
            Ok(Helper::Empty(_)) => Ok(LlmVec(Vec::new())), // 识别为空列表
            Err(e) => Err(serde::de::Error::custom(format!(
                "【XML 结构非法】既不是有效的 <item> 序列，也不是合法的空列表。详情: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    const CORRECT_DATA: &str = r#"<FilesChoiceResponseLlm>
  <all>false</all>
  <files>
    <item>3052935</item>
    <item>3036828</item>
    <item>3036831</item>
    <item>3036837</item>
    <item>3036834</item>
    <item>3036825</item>
    <item>3036843</item>
    <item>3036840</item>
  </files>
</FilesChoiceResponseLlm>"#;
    const CORRECT_EMPTY_DATA: &str = r#"<FilesChoiceResponseLlm>
  <all>true</all>
  <files />
</FilesChoiceResponseLlm>"#;
    const WRONG_DATA: &str = r#"<FilesChoiceResponseLlm>
  <all>false</all>
  <files>
    <file>我是错误返回的字符串</file>
  </files>
  </FilesChoiceResponseLlm>"#;

    use super::*;
    use crate::api::llm::tool::{LlmBool, LlmPrompt, LlmVec};
    use helper::LlmPrompt;
    use quick_xml::de::from_str;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, LlmPrompt, Serialize, Deserialize)]
    pub struct FilesChoiceResponseLlm {
        #[prompt("如果目的是选择所有的内容则设置为 true，否则为 false")]
        pub all: LlmBool,
        #[prompt("请注意这里对应的是提供的内容的reference_id字段")]
        pub files: LlmVec<i64>,
    }

    #[test]
    fn test() {
        let data = from_str::<FilesChoiceResponseLlm>(CORRECT_DATA);
        println!("Parsed data: {:?}", data);
        let data = from_str::<FilesChoiceResponseLlm>(CORRECT_EMPTY_DATA);
        println!("Parsed data: {:?}", data);
        let data = from_str::<FilesChoiceResponseLlm>(WRONG_DATA);
        println!("Parsed data: {:?}", data);
        println!()
    }
}
