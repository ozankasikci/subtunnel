//! Control channel message types for the subtunnel protocol.

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
    RegisterReq {
        protocol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subdomain: Option<String>,
    },
    RegisterResp {
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
    fn roundtrip_register_req() {
        let msg = ControlMessage::RegisterReq {
            protocol: "tcp".into(),
            subdomain: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"register_req","protocol":"tcp"}"#);
        let decoded: ControlMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn roundtrip_register_resp() {
        let msg = ControlMessage::RegisterResp {
            success: true,
            tunnel_id: "t_abc123".into(),
            subdomain: "a1b2c3d4".into(),
            message: "tunnel created".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(
            json,
            r#"{"type":"register_resp","success":true,"tunnel_id":"t_abc123","subdomain":"a1b2c3d4","message":"tunnel created"}"#
        );
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
