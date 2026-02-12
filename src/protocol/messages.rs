//! Control channel message types for the tunnelr protocol.
//!
//! All control messages are JSON-serialized and length-prefixed on the wire.

use serde::{Deserialize, Serialize};

/// All possible control messages exchanged between server and client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    /// Client authenticates with the server.
    Auth { token: String },

    /// Server responds to authentication.
    AuthResp {
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Client requests a new tunnel.
    TunnelReq {
        /// Protocol to tunnel (tcp for MVP).
        protocol: String,
        /// Optional requested remote port (server picks one if omitted).
        #[serde(skip_serializing_if = "Option::is_none")]
        remote_port: Option<u16>,
    },

    /// Server responds with the assigned tunnel info.
    TunnelResp {
        success: bool,
        /// Unique tunnel identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        tunnel_id: Option<String>,
        /// The public address clients can connect to.
        #[serde(skip_serializing_if = "Option::is_none")]
        public_addr: Option<String>,
        /// Assigned remote port.
        #[serde(skip_serializing_if = "Option::is_none")]
        remote_port: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },

    /// Heartbeat ping (sent by either side).
    Heartbeat,

    /// Heartbeat acknowledgement.
    HeartbeatAck,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_auth() {
        let msg = ControlMessage::Auth {
            token: "secret".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_tunnel_resp() {
        let msg = ControlMessage::TunnelResp {
            success: true,
            tunnel_id: Some("t_abc123".into()),
            public_addr: Some("server.example.com:12345".into()),
            remote_port: Some(12345),
            message: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_heartbeat() {
        let msg = ControlMessage::Heartbeat;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"heartbeat"}"#);
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }
}
