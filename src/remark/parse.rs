use crate::utils::data_structures::Map;
use std::borrow::Cow;

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DebugLocation<'a> {
    #[serde(borrow)]
    pub file: Cow<'a, str>,
    pub line: u32,
    pub column: u32,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RemarkArgString<'a> {
    #[serde(borrow)]
    pub string: Cow<'a, str>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RemarkArgCallee<'a> {
    #[serde(borrow)]
    pub callee: Cow<'a, str>,
    pub debug_loc: Option<DebugLocation<'a>>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RemarkArgCaller<'a> {
    #[serde(borrow)]
    pub caller: Cow<'a, str>,
    pub debug_loc: Option<DebugLocation<'a>>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RemarkArgReason<'a> {
    #[serde(borrow)]
    pub reason: Cow<'a, str>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum RemarkArg<'a> {
    #[serde(borrow)]
    String(RemarkArgString<'a>),
    Callee(RemarkArgCallee<'a>),
    Caller(RemarkArgCaller<'a>),
    Reason(RemarkArgReason<'a>),
    Other(Map<String, String>),
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MissedRemark<'a> {
    #[serde(borrow)]
    pub pass: Cow<'a, str>,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    pub debug_loc: Option<DebugLocation<'a>>,
    #[serde(borrow)]
    pub function: Cow<'a, str>,
    pub args: Vec<RemarkArg<'a>>,
}

#[derive(serde::Deserialize, Debug)]
pub enum Remark<'a> {
    #[serde(borrow)]
    Missed(MissedRemark<'a>),
    Passed {},
    Analysis {},
}
