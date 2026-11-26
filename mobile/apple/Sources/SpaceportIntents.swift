// ABOUTME: Exposes Spaceport's session, agent, and draft actions to Siri and Shortcuts.
// ABOUTME: Routes navigation to the app and reports when an agent's status was observed.
import AppIntents
import Foundation

struct ResumeSessionIntent: AppIntent {
    static let title: LocalizedStringResource = "Resume Session"
    static let description = IntentDescription("Open a saved session. Leave Session empty to resume your current or last session.")
    static let authenticationPolicy: IntentAuthenticationPolicy = .requiresLocalDeviceAuthentication
    static let openAppWhenRun = true

    @Parameter(title: "Session") var session: SpaceportSessionEntity?

    static var parameterSummary: some ParameterSummary { Summary("Resume \(\.$session)") }

    @MainActor func perform() async throws -> some IntentResult {
        try SpaceportShortcutRouter.shared.submit(.resume(session: session?.id))
        return .result()
    }
}

struct NextWaitingAgentIntent: AppIntent {
    static let title: LocalizedStringResource = "Open Next Waiting Agent"
    static let description = IntentDescription("Open the next agent waiting for you in your chosen, current, or last session.")
    static let authenticationPolicy: IntentAuthenticationPolicy = .requiresLocalDeviceAuthentication
    static let openAppWhenRun = true

    @Parameter(title: "Session") var session: SpaceportSessionEntity?

    static var parameterSummary: some ParameterSummary { Summary("Open the next waiting agent in \(\.$session)") }

    @MainActor func perform() async throws -> some IntentResult {
        try SpaceportShortcutRouter.shared.submit(.nextWaiting(session: session?.id))
        return .result()
    }
}

struct OpenAgentIntent: AppIntent {
    static let title: LocalizedStringResource = "Open Agent"
    static let description = IntentDescription("Open a saved agent in its session.")
    static let authenticationPolicy: IntentAuthenticationPolicy = .requiresLocalDeviceAuthentication
    static let openAppWhenRun = true

    @Parameter(title: "Agent") var agent: SpaceportAgentEntity

    static var parameterSummary: some ParameterSummary { Summary("Open \(\.$agent)") }

    @MainActor func perform() async throws -> some IntentResult {
        try SpaceportShortcutRouter.shared.submit(.openAgent(id: agent.id))
        return .result()
    }
}

struct GetAgentStatusIntent: AppIntent {
    static let title: LocalizedStringResource = "Get Agent Status"
    static let description = IntentDescription("Read an agent's last observed status and when Spaceport saw it.")
    static let authenticationPolicy: IntentAuthenticationPolicy = .requiresLocalDeviceAuthentication

    @Parameter(title: "Agent") var agent: SpaceportAgentEntity

    static var parameterSummary: some ParameterSummary { Summary("Get the last observed status of \(\.$agent)") }

    @MainActor func perform() async throws -> some IntentResult & ReturnsValue<String> & ProvidesDialog {
        let report = try statusReport(from: ProfileStore())
        return .result(value: report, dialog: "\(report)")
    }

    @MainActor func statusReport(from store: ProfileStore) throws -> String {
        guard let row = try store.catalog().systemAgent(id: agent.id) else { throw ProfileError.missingAgent }
        let status = PaneActivity(row.agent.status)?.title ?? "Unknown"
        let time = row.seenAt.formatted(date: .abbreviated, time: .shortened)
        return "\(row.agent.name) · \(status). Last observed \(time)."
    }
}

struct ComposePromptIntent: AppIntent {
    static let title: LocalizedStringResource = "Compose Prompt"
    static let description = IntentDescription("Open the composer. Choose a session or add draft text to review before sending.")
    static let authenticationPolicy: IntentAuthenticationPolicy = .requiresLocalDeviceAuthentication
    static let openAppWhenRun = true

    @Parameter(title: "Prompt") var prompt: String?
    @Parameter(title: "Session") var session: SpaceportSessionEntity?

    static var parameterSummary: some ParameterSummary { Summary("Compose \(\.$prompt) in \(\.$session)") }

    @MainActor func perform() async throws -> some IntentResult {
        try SpaceportShortcutRouter.shared.submit(.compose(session: session?.id, prompt: prompt ?? ""))
        return .result()
    }
}

struct SpaceportAppShortcuts: AppShortcutsProvider {
    static var appShortcuts: [AppShortcut] {
        AppShortcut(intent: ResumeSessionIntent(), phrases: ["Resume my session in \(.applicationName)"],
                    shortTitle: "Resume Session", systemImageName: "terminal")
        AppShortcut(intent: NextWaitingAgentIntent(), phrases: ["Open my next waiting agent in \(.applicationName)"],
                    shortTitle: "Next Waiting Agent", systemImageName: "bell")
        AppShortcut(intent: OpenAgentIntent(), phrases: ["Open an agent in \(.applicationName)",
                                                          "Open \(\.$agent) in \(.applicationName)"],
                    shortTitle: "Open Agent", systemImageName: "person.crop.circle")
        AppShortcut(intent: GetAgentStatusIntent(), phrases: ["Check an agent's status in \(.applicationName)",
                                                               "Check the status of \(\.$agent) in \(.applicationName)"],
                    shortTitle: "Get Agent Status", systemImageName: "waveform.path.ecg")
        AppShortcut(intent: ComposePromptIntent(), phrases: ["Compose a prompt in \(.applicationName)"],
                    shortTitle: "Compose Prompt", systemImageName: "square.and.pencil")
    }
}
