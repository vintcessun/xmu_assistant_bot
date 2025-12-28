use crate::abi::echo::{Echo, EchoPending};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub trait Data: Send + Sync + 'static + Serialize {}

pub struct ApiResponsePending<R> {
    pub echo: EchoPending,
    _marker: PhantomData<R>,
}

impl<R: ApiResponseTrait + for<'de> Deserialize<'de>> ApiResponsePending<R> {
    pub fn new(echo: Echo) -> Self {
        Self {
            echo: EchoPending::new(echo),
            _marker: PhantomData,
        }
    }

    pub async fn wait_echo(self) -> Result<R> {
        let response_str = self.echo.wait().await?;
        let response = serde_json::from_str::<R>(&response_str)?;
        Ok(response)
    }
}

pub trait ApiResponseTrait {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T: Data> {
    pub status: Status,
    pub retcode: u16,
    pub message: Option<String>,
    pub data: Option<T>,
    pub echo: Echo,
    pub wording: Option<String>,
    pub stream: Option<Stream>,
}

impl<T: Data> ApiResponseTrait for ApiResponse<T> {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Stream {
    StreamAction,
    NormalAction,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Async,
    Failed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMsgData {
    pub message_id: i64,
}

impl Data for SendMsgData {}

pub type SendMsgResponse = ApiResponse<SendMsgData>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ForwardMsgData {
    pub message_id: i64,
    pub res_id: Option<String>,
}

impl Data for ForwardMsgData {}

#[derive(Serialize, Deserialize, Debug)]
pub struct PokeData {}

impl Data for PokeData {}

pub type ForwardMsgResponse = ApiResponse<ForwardMsgData>;

pub type PokeResponse = ApiResponse<PokeData>;
