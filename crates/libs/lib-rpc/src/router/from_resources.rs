use crate::RpcResources;
use crate::router::Result;

pub trait FromResources {
    fn from_resources(rpc_resources: &RpcResources) -> Result<Self>
    where
        Self: Sized;
}
