use crate::register::RegisterBuilder;
use crate::route::Route;
use crate::support::triple::TripleRequestWrapper;
use bytes::BufMut;
use http::Request;
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http2::SendRequest;
use krpc_common::{KrpcMsg, RpcError};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct KrpcClient {
    route: Route,
}

impl KrpcClient {
    pub fn build(register_builder: RegisterBuilder) -> KrpcClient {
        let map = Arc::new(RwLock::new(HashMap::new()));
        let register = register_builder.init(map.clone());
        let cli = KrpcClient {
            route: Route::new(map, register),
        };
        return cli;
    }

    pub async fn invoke<Res>(&self, msg: KrpcMsg) -> Result<Res, RpcError>
    where
        Res: Send + Sync + Serialize + for<'a> Deserialize<'a> + Default,
    {
        let mut sender: SendRequest<Full<bytes::Bytes>> = self
            .route
            .get_socket_sender(&msg.class_name, &msg.version)
            .await
            .map_err(|e| RpcError::Client(e.to_string()))?;

        let mut buf = bytes::BytesMut::new();
        buf.put_u8(b'\0');
        buf.put_u8(b'\0');
        buf.put_u8(b'\0');
        buf.put_u8(b'\0');
        buf.put_u8(70);
        let mut tri = TripleRequestWrapper::get_request(msg.req);
        tri.encode(&mut buf);
        let req = Request::builder()
            .uri("/".to_owned() + &msg.class_name + "/" + &msg.method_name)
            .header("content-type", "application/grpc+proto")
            .body(Full::<bytes::Bytes>::from(buf.to_vec()))
            .map_err(|e| RpcError::Client(e.to_string()))?;
        let mut res = sender
            .send_request(req)
            .await
            .map_err(|e| RpcError::Client(e.to_string()))?;
        let res_data = res.frame()
        .await
        .unwrap()
        .map_err(|e| RpcError::Client(e.to_string()))?
        .into_data().unwrap();
        println!("sdasdad {:?}",res_data);
        let res: Result<String, RpcError> = serde_json::from_slice(
                res_data
                .as_ref(),
        )
        .map_err(|e| RpcError::Client(e.to_string()))?;
        let res: Result<Res, RpcError> = match res {
            Ok(data) => Ok(serde_json::from_slice(&data.as_bytes())
                .map_err(|e| RpcError::Client(e.to_string()))?),
            Err(err) => Err(err),
        };
        return res;
    }
}
