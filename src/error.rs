pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // protocol
    #[error("protocol connect: {0}")]
    ProtocolConnect(String),
    #[error("protocol timeout {endpoint} after {timeout_ms}ms")]
    ProtocolTimeout { endpoint: String, timeout_ms: u64 },
    #[error("protocol read {address}: {reason}")]
    ProtocolRead { address: String, reason: String },
    #[error("protocol write {address}: {reason}")]
    ProtocolWrite { address: String, reason: String },
    #[error("protocol decode: {0}")]
    ProtocolDecode(String),
    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    // inference
    #[error("model load: {0}")]
    ModelLoad(String),
    #[error("inference: {0}")]
    Inference(String),
    #[error("tensor shape: expected {expected:?} got {actual:?}")]
    TensorShape {
        expected: Vec<usize>,
        actual: Vec<usize>,
    },
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("preprocess: {0}")]
    Preprocess(String),
    #[error("postprocess: {0}")]
    Postprocess(String),
    // device
    #[error("device unreachable: {0}")]
    DeviceUnreachable(String),
    #[error("device calibration: {0}")]
    Calibration(String),
    #[error("device read-only: {0}")]
    ReadOnly(String),
    // pipeline
    #[error("pipeline stage {idx} failed: {src}")]
    StageFailed { idx: usize, src: Box<Error> },
    #[error("backpressure capacity {cap} exceeded")]
    Backpressure { cap: usize },
    #[error("pipeline cancelled")]
    Cancelled,
    // security
    #[error("tls: {0}")]
    Tls(String),
    #[error("auth: {0}")]
    Auth(String),
    #[error("authorization: {perm} denied for {res}")]
    Authorization { res: String, perm: String },
    #[error("certificate: {0}")]
    Certificate(String),
    #[error("replay attack nonce={nonce}")]
    Replay { nonce: String },
    // config
    #[error("config: {0}")]
    Config(String),
    #[error("missing field {field} in [{sec}]")]
    Missing { sec: String, field: String },
    // i/o
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde: {0}")]
    Serde(String),
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e.to_string())
    }
}

pub fn protocol_decode(msg: impl Into<String>) -> Error {
    Error::ProtocolDecode(msg.into())
}

pub fn protocol_connect(msg: impl Into<String>) -> Error {
    Error::ProtocolConnect(msg.into())
}

pub fn protocol_timeout(endpoint: String, ms: u64) -> Error {
    Error::ProtocolTimeout {
        endpoint,
        timeout_ms: ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_protocol_connect() {
        let e = Error::ProtocolConnect("refused".into());
        assert_eq!(e.to_string(), "protocol connect: refused");
    }

    #[test]
    fn display_protocol_timeout() {
        let e = Error::ProtocolTimeout {
            endpoint: "10.0.0.1:502".into(),
            timeout_ms: 1000,
        };
        assert!(e.to_string().contains("10.0.0.1:502"));
    }

    #[test]
    fn display_tensor_shape() {
        let e = Error::TensorShape {
            expected: vec![1, 3, 224, 224],
            actual: vec![1, 3, 128, 128],
        };
        assert!(e.to_string().contains("224"));
    }

    #[test]
    fn display_cancelled() {
        let e = Error::Cancelled;
        assert_eq!(e.to_string(), "pipeline cancelled");
    }

    #[test]
    fn display_config() {
        let e = Error::Config("bad value".into());
        assert_eq!(e.to_string(), "config: bad value");
    }

    #[test]
    fn helper_functions() {
        let e = protocol_decode("bad header");
        assert!(matches!(e, Error::ProtocolDecode(_)));

        let e = protocol_connect("timeout");
        assert!(matches!(e, Error::ProtocolConnect(_)));

        let e = protocol_timeout("10.0.0.1:502".into(), 5000);
        assert!(matches!(e, Error::ProtocolTimeout { .. }));
    }

    #[test]
    fn result_type_alias() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);
    }

    #[test]
    fn from_serde_json() {
        let e = Error::from(serde_json::from_str::<i32>("not-a-number").unwrap_err());
        assert!(matches!(e, Error::Serde(_)));
    }

    #[test]
    fn from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e: Error = io_err.into();
        assert!(matches!(e, Error::Io(_)));
    }
}
