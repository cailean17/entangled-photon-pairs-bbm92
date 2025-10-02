pub mod api_structures;
use crate::{api_controller::api_structures::TelemetryWebSocketRequestMessage, config::Config};
use reqwest_websocket::{Message, RequestBuilderExt, WebSocket};
use serde_json::{Value, json};
use futures_util::{SinkExt, TryStreamExt};

#[derive(Debug)]
pub struct ApiController {
    control_plane_api: String,
    web_socket_url: Option<String>,
    control_plane_client: reqwest::Client,
    web_socket_client: Option<WebSocket>,
}

impl ApiController {
    pub fn new(control_plane_api: String) -> Self {
        Self {
            control_plane_api: control_plane_api,
            web_socket_url: None,
            control_plane_client: reqwest::Client::new(),
            web_socket_client: None,
        }
    }

    pub fn set_web_socket_url(&mut self, client_web_socket_url: Option<String>) {
        self.web_socket_url = client_web_socket_url;
    }

    fn set_web_socket_client(&mut self, web_socket_client: Option<WebSocket>) {
        self.web_socket_client = web_socket_client;
    }

    pub async fn control_plane_start_new_connection(
        &mut self,
    ) -> Result<api_structures::ControlPlaneStartConnectionResponse, Box<dyn std::error::Error>>
    {
        let req_body: Value = json!({});
        let res: api_structures::ControlPlaneStartConnectionResponse = serde_json::from_str(
            &self
                .control_plane_client
                .post(format!("{}/StartConnection", &self.control_plane_api))
                .json(&req_body)
                .send()
                .await?
                .text()
                .await?,
        )?;
        match res.status_code {
            200 => {
                self.set_web_socket_url(Some(
                    res.body.websocket_client_endpoint.clone()
                ));
                println!("{:?}", res);
                Ok(res)
            }

            _ => Err(format!(
                "StartConnection failed with status code: {}",
                res.status_code
            )
            .into()),
        }
    }

    pub async fn web_socket_join_connection(&mut self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        match &self.web_socket_url {
            Some(url) => {
                let url_with_query_params = url.clone()
                  + "&spad_a_channel_id="
                  + &config.get_spad_a_channel_id().to_string()
                  + "&spad_b_channel_id="
                  + &config.get_spad_b_channel_id().to_string()
                  + "&basis_bit_channel_id="
                  + &config.get_basis_bit_channel_id().to_string();
                let res = reqwest::Client::default()
                    .get(url_with_query_params)
                    .upgrade()
                    .send()
                    .await?
                    .into_websocket()
                    .await?;
                self.set_web_socket_client(Some(res));

                Ok(())
            },
            None => {
                return Err(
                    "Attempted to start a websocket connection with empty websocket url".into(),
                );
            }
        }
    }

    pub async fn web_socket_send(&mut self, message: TelemetryWebSocketRequestMessage) -> Result<String, Box<dyn std::error::Error>>{
      match &mut self.web_socket_client {
        Some(client) => {
          let json = serde_json::to_string(&message).unwrap();
          println!("{}", json.as_str());
          client.send(Message::Text(json)).await?;
          return Ok("DONE".to_string());
        }
        None => {
          return Err("Attempted to read from non-existent websocket conneciton".into());
        }

      }
    }
    pub async fn web_socket_read(&mut self) -> Result<Option<reqwest_websocket::Message>, Box<dyn std::error::Error>> {
      match &mut self.web_socket_client {
        Some(client) => {
          while let Some(message) = client.try_next().await? {
              match message {
                Message::Text(_) => {
                return Ok(Some(message))
                },
                _ => {
                  continue;
                }
              }
          }

          Ok(None)
        },
        None => {
          return Err("Attempted to read from non-existent websocket conneciton".into());
        }
      }
    }


}
