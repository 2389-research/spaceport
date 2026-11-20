// ABOUTME: Exercises recorder refusal with observed telemetry shapes and documented system sampling identities.
// ABOUTME: Keeps ordinary OS kernel providers distinct from CPU profile sessions.
use super::*;

const TELEMETRY: &str = r#"Logger Name           : Mellanox-Kernel
Log File Mode         : Circular PersistOnHybridShutdown
Trace Flags           : 83793ec3-e8a3-4801-becf-5e921f79f06e:0x7fffffff:0x3

Logger Name           : Admin_PS_Provider
Log File Mode         : PersistOnHybridShutdown
Trace Flags           : "Microsoft-Windows-Kernel-Process":0x70:0xff
"#;

#[test]
fn observed_os_telemetry_without_cpu_recording_allows_preflight() -> Result<()> {
    ensure_available("WPR is not recording\r\n", TELEMETRY)
}

#[test]
fn observed_security_logger_has_a_present_empty_trace_flags_field() -> Result<()> {
    ensure_available(
        "WPR is not recording",
        "Logger Name : Eventlog-Security\nLog File Mode : NonStoppable Secure PersistOnHybridShutdown IndependentSession NoPerProcessorBuffering\nTrace Flags :\n",
    )
}

#[test]
fn observed_guid_providers_allow_omitted_keywords_and_levels() -> Result<()> {
    for provider in [
        "b1ad9b49-051a-4896-ae24-ebc9b8676e76::0x5",
        "2434b2ae-fcfd-4b69-9ab4-e58a0dd56565:0x3",
        "1b1d4ff4-f27b-4c99-8bd7-da8f1a74051a",
    ] {
        ensure_available(
            "WPR is not recording",
            &format!(
                "Logger Name : Observed telemetry\nLog File Mode : Circular\nTrace Flags : {provider}\n"
            ),
        )?;
    }
    Ok(())
}

#[test]
fn refuses_default_named_and_system_profile_recordings() {
    assert!(ensure_available("WPR recording is in progress", TELEMETRY).is_err());
    for logger in [
        "Logger Name : WPR_initiated_CPU_External\nLog File Mode : Sequential\nTrace Flags :\n",
        "Logger Name : NT Kernel Logger\nLog File Mode : Sequential\nTrace Flags : PROC_THREAD+LOADER+PROFILE\n",
        "Logger Name : Other\nLog File Mode : SystemLogger\nTrace Flags : 0x01000000\n",
        "Logger Name : Other\nLog File Mode : Sequential\nTrace Flags : bfeb0324-1cee-496f-a409-2ac2b48a6322:0x1:0x5\n",
    ] {
        assert!(ensure_available("WPR is not recording", logger).is_err());
    }
}

#[test]
fn refuses_missing_truncated_or_unclassified_logger_state() {
    for loggers in [
        "",
        "Logger Name : Other\nLog File Mode : Sequential\n",
        "Logger Name : Other\nLog File Mode : Sequential\nTrace Flags : UNKNOWN_KERNEL_FLAG\n",
    ] {
        assert!(ensure_available("WPR is not recording", loggers).is_err());
    }
    assert!(ensure_available("unrecognized WPR output", TELEMETRY).is_err());
}
