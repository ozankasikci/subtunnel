//! Control channel message types for the tunnelr protocol.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    Auth {
        token: String,
    },
    AuthResp {
        success: bool,
        message: String,
    },
    TunnelReq {
        protocol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subdomain: Option<String>,
    },
    TunnelResp {
        success: bool,
        tunnel_id: String,
        subdomain: String,
        message: String,
    },
    Heartbeat,
    HeartbeatAck,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_auth() {
        let msg = ControlMessage::Auth { token: "secret".into() };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_tunnel_req() {
        let msg = ControlMessage::TunnelReq { protocol: "tcp".into(), subdomain: None };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_tunnel_resp() {
        let msg = ControlMessage::TunnelResp {
            success: true,
            tunnel_id: "t_abc123".into(),
            subdomain: "a1b2c3d4".into(),
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
}
