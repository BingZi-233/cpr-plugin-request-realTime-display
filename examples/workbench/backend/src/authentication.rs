use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::frontend_authentication::{
        FrontendAuthenticationIdentifier, FrontendAuthenticationRequest,
        FrontendAuthenticationResult,
    },
    client::{Empty, TypedCall, TypedReply},
};

pub(crate) async fn frontend_identifier(
    _call: TypedCall<Empty>,
) -> Result<TypedReply<FrontendAuthenticationIdentifier>, PluginFault> {
    Ok(TypedReply::new(FrontendAuthenticationIdentifier {
        identifier: "request-realtime-display".to_owned(),
    }))
}

pub(crate) async fn frontend_authenticate(
    evidence: &EvidenceLog,
    call: TypedCall<FrontendAuthenticationRequest>,
) -> Result<TypedReply<FrontendAuthenticationResult>, PluginFault> {
    let (event, result) = if call.request.authorization == "RequestRealtimeDisplay demo" {
        (
            "demo_identity_authenticated",
            FrontendAuthenticationResult::Authenticated {
                principal: "request-realtime-display-demo-user".to_owned(),
            },
        )
    } else if call
        .request
        .authorization
        .starts_with("RequestRealtimeDisplay ")
    {
        (
            "invalid_demo_identity_rejected",
            FrontendAuthenticationResult::Rejected {},
        )
    } else {
        (
            "unrelated_identity_delegated",
            FrontendAuthenticationResult::NotMatched {},
        )
    };
    evidence.record(EvidenceInput::passed("frontend_authentication", event));
    Ok(TypedReply::new(result))
}
