// ABOUTME: Presents saved hosts, sessions and observed agents to Siri and Shortcuts.
// ABOUTME: Resolves each agent against the full target recorded in the current catalog.
import AppIntents
import CryptoKit
import Foundation

protocol SpaceportCatalogEntity: AppEntity {
    var name: String { get }
    var detail: String { get }
}

extension SpaceportCatalogEntity {
    var displayRepresentation: DisplayRepresentation {
        DisplayRepresentation(title: "\(name)", subtitle: "\(detail)")
    }
}

protocol SpaceportCatalogQuery: EntityStringQuery where Entity: SpaceportCatalogEntity {
    func choices(in catalog: HostCatalog) -> [Entity]
}

extension SpaceportCatalogQuery {
    func suggestedEntities() async throws -> [Entity] {
        choices(in: try await ProfileStore().catalog())
    }

    func entities(for identifiers: [Entity.ID]) async throws -> [Entity] {
        let wanted = Set(identifiers)
        return try await suggestedEntities().filter { wanted.contains($0.id) }
    }

    func entities(matching string: String) async throws -> [Entity] {
        try await suggestedEntities().filter {
            string.isEmpty || $0.name.localizedCaseInsensitiveContains(string) ||
                $0.detail.localizedCaseInsensitiveContains(string)
        }
    }
}

struct SpaceportHostEntity: SpaceportCatalogEntity {
    static let typeDisplayRepresentation: TypeDisplayRepresentation = "Host"
    static let defaultQuery = HostQuery()
    let id: UUID
    let name: String
    let detail: String

    struct HostQuery: SpaceportCatalogQuery {
        typealias Entity = SpaceportHostEntity
        func choices(in catalog: HostCatalog) -> [SpaceportHostEntity] { catalog.systemHosts }
    }
}

struct SpaceportSessionEntity: SpaceportCatalogEntity {
    static let typeDisplayRepresentation: TypeDisplayRepresentation = "Session"
    static let defaultQuery = SessionQuery()
    let id: UUID
    let name: String
    let detail: String

    struct SessionQuery: SpaceportCatalogQuery {
        typealias Entity = SpaceportSessionEntity
        func choices(in catalog: HostCatalog) -> [SpaceportSessionEntity] { catalog.systemSessions }
    }
}

struct SpaceportAgentEntity: SpaceportCatalogEntity {
    static let typeDisplayRepresentation: TypeDisplayRepresentation = "Agent"
    static let defaultQuery = AgentQuery()
    let id: String
    let name: String
    let detail: String
    let status: String
    let observedAt: Date

    struct AgentQuery: SpaceportCatalogQuery {
        typealias Entity = SpaceportAgentEntity
        func choices(in catalog: HostCatalog) -> [SpaceportAgentEntity] { catalog.systemAgents }
    }

    fileprivate static func identifier(for row: HomeAgent) -> String {
        let agent = row.agent
        // Length-prefix each field so names and daemon IDs can contain arbitrary separators.
        let fields = [row.profile.id.uuidString, agent.bootId, agent.workspaceId, agent.tabId, agent.paneId, agent.name]
        let input = fields.map { "\($0.utf8.count):\($0)" }.joined()
        return SHA256.hash(data: Data(input.utf8)).map { String(format: "%02x", $0) }.joined()
    }
}

extension HostCatalog {
    var systemHosts: [SpaceportHostEntity] {
        hosts.map { .init(id: $0.id, name: $0.title, detail: $0.listHeading(in: hosts)) }
    }

    var systemSessions: [SpaceportSessionEntity] {
        sessions.compactMap { session in
            guard let host = hosts.first(where: { $0.id == session.hostID }) else { return nil }
            return .init(id: session.id, name: session.name, detail: host.listHeading(in: hosts))
        }
    }

    var systemAgents: [SpaceportAgentEntity] {
        observedAgents.map { row in
            let detail = [row.agent.paneId, row.agent.location, row.profile.session, row.host.listHeading(in: hosts)]
                .joined(separator: " › ")
            return .init(id: SpaceportAgentEntity.identifier(for: row), name: row.agent.name, detail: detail,
                         status: row.agent.status, observedAt: row.seenAt)
        }
    }

    func systemAgent(id: String) -> HomeAgent? {
        observedAgents.first { SpaceportAgentEntity.identifier(for: $0) == id }
    }

    var lastOpenedSession: SavedSession? {
        let hostIDs = Set(hosts.map(\.id))
        return sessions.filter { $0.lastOpened != nil && hostIDs.contains($0.hostID) }.min {
            if $0.lastOpened == $1.lastOpened { return $0.id.uuidString < $1.id.uuidString }
            return ($0.lastOpened ?? .distantPast) > ($1.lastOpened ?? .distantPast)
        }
    }
}
