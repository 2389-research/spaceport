// ABOUTME: Resolves system actions against the current local SSH catalog.
// ABOUTME: Delivers a foreground action once, including during a cold launch.
import Foundation

enum SpaceportShortcut: Equatable {
    case resume(session: UUID?)
    case nextWaiting(session: UUID?)
    case widgetNextWaiting(destination: UUID)
    case openAgent(id: String)
    case compose(session: UUID?, prompt: String)

    struct Destination {
        let profile: HostProfile
        let agent: SeenAgent?
        let action: SessionShortcutAction
    }

    func destination(in catalog: HostCatalog, active: HostProfile?) throws -> Destination {
        let id: UUID?
        let action: SessionShortcutAction
        switch self {
        case let .openAgent(reference):
            guard let row = catalog.systemAgent(id: reference) else { throw ShortcutFailure.missingAgent }
            return Destination(profile: row.profile, agent: row.agent, action: .revealTerminal)
        case let .resume(session):
            id = session
            action = .revealTerminal
        case let .nextWaiting(session):
            id = session
            action = .nextWaiting
        case let .widgetNextWaiting(destination):
            guard let session = catalog.widgetSession(for: destination) else { throw ShortcutFailure.changedWidgetSession }
            id = session
            action = .nextWaiting
        case let .compose(session, prompt):
            id = session
            action = .compose(prompt)
        }

        if let id {
            guard let profile = catalog.profiles.first(where: { $0.id == id }) else { throw ShortcutFailure.missingSession }
            return Destination(profile: profile, agent: nil, action: action)
        }
        if let active, catalog.profiles.contains(active) {
            return Destination(profile: active, agent: nil, action: action)
        }
        guard let session = catalog.lastOpenedSession, let profile = catalog.profile(session) else {
            throw ShortcutFailure.noRecentSession
        }
        return Destination(profile: profile, agent: nil, action: action)
    }
}

enum ShortcutFailure: LocalizedError {
    case busy, changedWidgetSession, editing, missingAgent, missingSession, noRecentSession

    var errorDescription: String? {
        switch self {
        case .busy: "Spaceport is opening another shortcut. Try again once it opens."
        case .changedWidgetSession: "The widget’s session has changed. Choose a saved session in Spaceport."
        case .editing: "Finish editing your saved sessions, then run the shortcut again."
        case .missingAgent: "That agent is no longer in the saved roster. Choose an agent in Spaceport."
        case .missingSession: "That session is no longer saved on this device."
        case .noRecentSession: "Open a session in Spaceport first, or choose a saved session in the shortcut."
        }
    }
}

extension SceneSessions {
    func openShortcut(_ request: SpaceportShortcut, in catalog: HostCatalog) throws -> SessionPresentation {
        guard current?.connection.pendingShortcut == nil else { throw ShortcutFailure.busy }
        let destination = try request.destination(in: catalog, active: current?.connection.profile)
        let presentation = open(destination.profile, agent: destination.agent)
        presentation.connection.requestShortcut(destination.action, presentation: presentation.id)
        return presentation
    }
}

@MainActor
final class SpaceportShortcutRouter {
    static let shared = SpaceportShortcutRouter()
    private var scenes: [(id: UUID, handle: (SpaceportShortcut) throws -> Void)] = []
    private var pending: SpaceportShortcut?

    func submit(_ request: SpaceportShortcut) throws {
        if let scene = scenes.last {
            try scene.handle(request)
        } else {
            guard pending == nil else { throw ShortcutFailure.busy }
            pending = request
        }
    }

    func activate(_ id: UUID, handler: @escaping (SpaceportShortcut) throws -> Void) throws {
        deactivate(id)
        scenes.append((id, handler))
        if let request = pending {
            pending = nil
            try handler(request)
        }
    }

    func deactivate(_ id: UUID) {
        scenes.removeAll { $0.id == id }
    }
}
