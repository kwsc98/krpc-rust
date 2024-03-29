use examples::ReqDto;
use examples::TestServerRpc;
use krpc_core::{
    client::KrpcClient,
    register::{RegisterBuilder, RegisterType},
};
use lazy_static::lazy_static;
use tracing::info;

lazy_static! {
    static ref CLI: KrpcClient = KrpcClient::build(RegisterBuilder::new(
        &format!("127.0.0.1:{}", "2181"),
        "default",
        RegisterType::ZooKeeper,
    ));
}

#[tokio::main(worker_threads = 512)]
async fn main() {
    let de = TestServerRpc::new(&CLI);
    krpc_common::init_log();
    let client = de;
    let res = client
        .do_run1(
            ReqDto {
                str: "client say hello 1".to_string(),
            },
            ReqDto {
                str: "client say hello 2".to_string(),
            },
        )
        .await;
    info!("{:?}", res);
    let res = client
        .doRun2(ReqDto {
            str: "client say hello 2".to_string(),
        })
        .await;
    info!("{:?}", res);
}
