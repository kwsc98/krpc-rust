#[macro_export]
macro_rules! krpc_server {
    (
    $package:expr,
    $name:ident,
    $version:expr,
    $(async fn $method:ident (&$self:ident $(,$req:ident : $reqType:ty)* ) -> Result<$resType:ty>  { $($code:tt)* })*) => {
        impl $name {
            $(
                #[allow(non_snake_case)]
                async fn $method (&$self $(,$req : $reqType)* ) -> Result<$resType,krpc_common::RpcError> { $($code)* }
            )*

            async fn prv_invoke (&self, mut param : krpc_common::KrpcMsg) -> krpc_common::KrpcMsg {
                $(if &param.method_name[..] == stringify!($method) {
                    let req = &param.req;
                    let mut idx = 0;
                    $(
                        let result : Result<$reqType,_>  = serde_json::from_slice(req[idx].as_bytes());
                        if let Err(err) = result {
                            param.res = Err(krpc_common::RpcError::Server(err.to_string()));
                            return param;
                        }
                        let $req : $reqType = serde_json::from_slice(req[idx].as_bytes()).unwrap();
                        idx += 1;
                    )*
                    let res = self.$method(
                        $(
                            $req,
                        )*
                    ).await;
                    param.res = match res {
                        Ok(res) => {
                            let res = serde_json::to_string(&res);
                            match res {
                                Ok(res) => Ok(res),
                                Err(err) => Err(krpc_common::RpcError::Server(err.to_string()))
                            }
                        },
                        Err(info) => Err(info)
                    };
                    return param;
                })*
                param.res = Err(krpc_common::RpcError::Server(format!("not find method by {}",param.method_name)));
                return param;
            }
        }
        impl krpc_common::RpcServer for $name {
            fn invoke (&self, param : krpc_common::KrpcMsg) -> krpc_common::KrpcFuture<krpc_common::KrpcMsg> {
                let rpc = self.clone();
                Box::pin(async move {rpc.prv_invoke(param).await})
            }
            fn get_info(&self) -> (&str , &str , Option<&str> , Vec<String>) {
               let mut methods = vec![];
               $(
                  methods.push(stringify!($method).to_string());
               )*
               ($package ,stringify!($name) , $version ,methods)
            }
        }
    }
}

#[macro_export]
macro_rules! krpc_client {
    (
    $cli:ident,
    $package:expr,
    $name:ident,
    $version:expr,
    $(async fn $method:ident (&$self:ident $(,$req:ident : $reqType:ty)* ) -> Result<$resType:ty> )*) => {
        impl $name {
            $(
                #[allow(non_snake_case)]
                async fn $method (&$self $(,$req : $reqType)*) -> Result<$resType,krpc_common::RpcError> {
                    let mut req_vec : Vec<String> = vec![];
                    $(
                        let mut res_str = serde_json::to_string(&$req);
                        if let Err(err) = res_str {
                            return Err(krpc_common::RpcError::Client(err.to_string()));
                        }
                        req_vec.push(res_str.unwrap());
                    )*
                    let version : Option<&str> = $version;
                    let msg = krpc_common::KrpcMsg::new(
                        krpc_common::get_uuid(),
                        version.map(|e|e.to_string()),
                        $package.to_owned() + "." + stringify!($name),
                        stringify!($method).to_string(),
                        req_vec,
                        Err(krpc_common::RpcError::Null)
                    );
                    let res : Result<$resType,krpc_common::RpcError> = $cli.invoke::<$resType>(msg).await;
                    return res;
                }
            )*
        }
    }
}