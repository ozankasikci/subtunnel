//! Control channel message types for the tunnelr protocol.
//!
//! All control messages are JSON-serialized and length-prefixed on the wire.
//! The enum is tagged via `serde(tag = "type")` so each variant carries a
//! `"type"` discriminator field in the JSON representation.

use serde::{Deserialize, Serialize};

/// All possible control messages exchanged between server and client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    /// Client authenticates with the server using a bearer token.
    Auth {
        /// Opaque authentication token.
        token: String,
    },

    /// Server responds to an authentication attempt.
    AuthResp {
        /// Whether authentication succeeded.
        success: bool,
        /// Human-readable status or error description.
        message: String,
    },

    /// Client requests a new tunnel be created.
    TunnelReq {
        /// Protocol to tunnel (`"tcp"` for MVP).
        protocol: String,
        /// Optional requested remote port; the server picks one if omitted.
        #[serde(skip_serializing_if = "Option::is_none")]
        remote_port: Option<u16>,
    },

    /// Server responds with assigned tunnel metadata.
    TunnelResp {
        /// Whether tunnel creation succeeded.
        success: bool,
        /// Unique tunnel identifier (e.g. `"t_abc123"`).
        tunnel_id: String,
        /// The public port assigned to this tunnel.
        remote_port: u16,
        /// Human-readable status or error description.
        message: String,
    },

    /// Heartbeat ping — sent periodically by either side to keep the
    /// connection alive and detect failures.
    Heartbeat,

    /// Heartbeat acknowledgement — the peer replies with this.
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
    fn roundtrip_auth_resp() {
        let msg = ControlMessage::AuthResp {
            success: true,
            message: "welcome".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_tunnel_req() {
        let msg = ControlMessage::TunnelReq {
            protocol: "tcp".into(),
            remote_port: Some(9090),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_tunnel_req_no_port() {
        let msg = ControlMessage::TunnelReq {
            protocol: "tcp".into(),
            remote_port: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("remote_port"));
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_tunnel_resp() {
        let msg = ControlMessage::TunnelResp {
            success: true,
            tunnel_id: "t_abc123".into(),
            remote_port: 12345,
            message: "tunnel created".into(),
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

    #[test]
    fn roundtrip_heartbeat_ack() {
        let msg = ControlMessage::HeartbeatAck;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"heartbeat_ack"}"#);
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn json_tag_format() {
        let msg = ControlMessage::Auth { token: "t".into() };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"auth""#));
    }
}
