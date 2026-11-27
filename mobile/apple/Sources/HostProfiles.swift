// ABOUTME: Stores SSH hosts and their Herdr sessions in a single local catalog.
// ABOUTME: Migrates flat session profiles while retaining session IDs and endpoint trust.
import Combine
import Foundation

struct SSHAddress: Codable, Hashable {
    var hostname: String
    var port: UInt16
    var username: String

    var endpoint: String {
        let host = hostname.contains(":") ? "[\(hostname)]" : hostname
        return host + (port == 22 ? "" : ":\(port)")
    }
    var destination: String { "\(username)@\(endpoint)" }
    var pinAccount: String { "host/[\(hostname.lowercased())]:\(port)" }

    static func parse(_ text: String) throws -> Self {
        let parts = text.trimmingCharacters(in: .whitespacesAndNewlines)
            .split(separator: "@", omittingEmptySubsequences: false)
        guard parts.count == 2 else { throw ProfileError.invalidAddress }
        let endpoint = parts[1]
        let host: Substring
        let port: Substring
        if endpoint.first == "[" {
            guard let end = endpoint.firstIndex(of: "]") else { throw ProfileError.invalidAddress }
            host = endpoint[endpoint.index(after: endpoint.startIndex)..<end]
            guard host.contains(":") else { throw ProfileError.invalidAddress }
            let suffix = endpoint[endpoint.index(after: end)...]
            guard suffix.isEmpty || suffix.first == ":" else { throw ProfileError.invalidAddress }
            port = suffix.isEmpty ? "22" : suffix.dropFirst()
        } else {
            let fields = endpoint.split(separator: ":", omittingEmptySubsequences: false)
            guard fields.count == 1 || fields.count == 2 else { throw ProfileError.invalidAddress }
            host = fields[0]
            port = fields.count == 2 ? fields[1] : "22"
        }
        let username = parts[0]
        guard !username.isEmpty, !host.isEmpty, host.utf8.count <= 255, username.utf8.count <= 255,
              !host.contains(where: { "/[]".contains($0) }),
              ![host, username].contains(where: { $0.unicodeScalars.contains(where: { CharacterSet.whitespacesAndNewlines.union(.controlCharacters).contains($0) }) })
        else { throw ProfileError.invalidAddress }
        guard !port.isEmpty, port.utf8.allSatisfy({ (48...57).contains($0) }),
              let number = UInt16(port), number > 0 else { throw ProfileError.invalidPort }
        return Self(hostname: String(host), port: number, username: String(username))
    }
}

// This is both the connection target and the wire shape of pre-catalog preferences.
struct HostProfile: Codable, Hashable, Identifiable {
    var id: UUID
    var hostname: String
    var port: UInt16
    var username: String
    var session: String
    var login: LoginMethod

    var address: SSHAddress { SSHAddress(hostname: hostname, port: port, username: username) }
    var pinAccount: String { address.pinAccount }

    static func make(id: UUID = UUID(), hostname: String, port: String, username: String,
                     session: String, login: LoginMethod = .phoneKey) throws -> Self {
        let fields = [hostname, username, session].map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        guard fields.allSatisfy({ !$0.isEmpty && !$0.contains(where: { $0.isWhitespace || $0.isNewline }) }) else {
            throw ProfileError.missingTarget
        }
        guard let port = UInt16(port), port > 0 else { throw ProfileError.invalidPort }
        let entered = fields[0]
        let hostname = entered.hasPrefix("[") && entered.hasSuffix("]") && entered.contains(":")
            ? String(entered.dropFirst().dropLast()) : entered
        return Self(id: id, hostname: hostname, port: port, username: fields[1], session: fields[2], login: login)
    }
}

enum LoginMethod: String, Codable, CaseIterable {
    case phoneKey, password
    var title: String { self == .phoneKey ? "Phone Key" : "Password" }
}

enum ProfileError: LocalizedError {
    case missingTarget, invalidPort, invalidAddress, duplicateSession, duplicateHost, missingHost, missingAgent, tooManyMarks, invalidCatalog
    var errorDescription: String? {
        switch self {
        case .missingTarget: "Enter a host, account, and session without spaces."
        case .invalidPort: "Enter an SSH port from 1 to 65535."
        case .invalidAddress: "Enter user@host or user@host:port. Put IPv6 addresses in brackets."
        case .duplicateSession: "This host already has a session with that name."
        case .duplicateHost: "A host already uses this address and sign-in method."
        case .missingHost: "This host or session is no longer saved."
        case .missingAgent: "This agent has changed. Open its session to refresh it."
        case .tooManyMarks: "This session has reached the limit for marked agents. Unpin or wake an agent first."
        case .invalidCatalog: "Spaceport could not read the saved SSH catalog."
        }
    }
}

struct SavedHost: Codable, Equatable, Identifiable {
    let id: UUID
    var address: SSHAddress
    var login: LoginMethod
    var name: String?

    var title: String { name ?? address.hostname }

    func listHeading(in hosts: [SavedHost]) -> String {
        var parts = [address.endpoint]
        if let name, !name.isEmpty, name != address.hostname { parts.insert(name, at: 0) }
        let peers = hosts.filter { $0.id != id && $0.address.endpoint == address.endpoint }
        if peers.contains(where: { $0.address.username != address.username }) { parts.append("Account " + address.username) }
        if peers.contains(where: { $0.address == address }) { parts.append(login.title) }
        return parts.joined(separator: " · ")
    }

    func profile(session: SavedSession) -> HostProfile {
        HostProfile(id: session.id, hostname: address.hostname, port: address.port,
                    username: address.username, session: session.name, login: login)
    }
}

struct SeenAgent: Hashable, Codable {
    let bootId: String
    let workspaceId: String
    let tabId: String
    let paneId: String
    let name: String
    let status: String
    let location: String
    var stateChangeSeq: UInt64?
    var changeObservedAt: Date?
    var idleOnsetObserved: Bool?
    var terminalTitle: String?
    var agentIdentity: String?
    var workspaceLabelBytes: Int?
    var paneTitle: String?

    var marks: AgentMarks?

    var locationLabels: (workspace: String, tab: String) {
        let separator = " › "
        let bytes = location.utf8
        if let length = workspaceLabelBytes, length >= 0,
           let end = bytes.index(bytes.startIndex, offsetBy: length, limitedBy: bytes.endIndex),
           bytes[end...].starts(with: separator.utf8) {
            return (String(decoding: bytes[..<end], as: UTF8.self),
                    String(decoding: bytes[end...].dropFirst(separator.utf8.count), as: UTF8.self))
        }
        // Older observations only stored the joined path. Preserve ambiguous labels verbatim.
        let parts = location.components(separatedBy: separator)
        return parts.count == 2 ? (parts[0], parts[1]) : (location, "")
    }

    static let cacheLimit = 32
    static let markLimit = 512

    static func cache(_ rows: some Sequence<SeenAgent>) -> [SeenAgent] {
        var saved: [SeenAgent] = []
        var paneIDs = Set<String>()
        var markedCount = 0
        for row in rows {
            // Retain the ordinary attention-first roster, plus agents the user explicitly marked.
            guard saved.count < cacheLimit || (row.marks != nil && markedCount < markLimit) else { continue }
            // A shortened ID or name could resolve to the wrong agent after reconnecting.
            let identity = [row.bootId, row.workspaceId, row.tabId, row.paneId, row.name]
            guard identity.allSatisfy({ !$0.isEmpty && $0.utf8.count <= 256 }),
                  row.status.utf8.count <= 32, row.location.utf8.count <= 512,
                  paneIDs.insert(row.paneId).inserted else { continue }
            saved.append(row)
            if row.marks != nil { markedCount += 1 }
        }
        return saved
    }
}

struct SessionStatusSnapshot: Codable, Equatable {
    let seenAt: Date
    let blocked: Int
    let working: Int
    let hasIdle: Bool
    let location: [String]
    var agents: [SeenAgent]?
    // Older snapshots counted all panes and may retain only part of the named roster.
    var agentCounts: AgentCounts?
}

struct SavedSession: Codable, Equatable, Identifiable {
    let id: UUID
    var hostID: UUID
    var name: String
    var lastOpened: Date?
    var lastStatus: SessionStatusSnapshot?
    private(set) var widgetDestinationOverride: UUID?

    var widgetDestination: UUID { widgetDestinationOverride ?? id }

    mutating func clearObservationForChangedTarget() {
        lastOpened = nil
        lastStatus = nil
        widgetDestinationOverride = UUID()
    }
}

struct HostCatalog: Codable {
    var hosts: [SavedHost] = []
    var sessions: [SavedSession] = []

    var profiles: [HostProfile] { sessions.compactMap { profile($0) } }
    var agentWidgetSnapshot: AgentWidgetSnapshot? {
        guard let session = lastOpenedSession, let status = session.lastStatus, let counts = status.agentCounts
        else { return nil }
        return AgentWidgetSnapshot(destinationID: session.widgetDestination, counts: counts, observedAt: status.seenAt)
    }

    func widgetSession(for destination: UUID) -> UUID? {
        sessions.first { $0.widgetDestination == destination && $0.lastStatus?.agentCounts != nil }?.id
    }

    var observedAgents: [HomeAgent] {
        let hostsByID = Dictionary(hosts.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
        let rows = sessions.flatMap { session -> [HomeAgent] in
            guard let host = hostsByID[session.hostID], let snapshot = session.lastStatus else { return [] }
            return HomeAgent.rows(from: snapshot, host: host, profile: host.profile(session: session))
        }
        return rows.sorted { left, right in
            if left.section != right.section { return left.section.rawValue < right.section.rawValue }
            return left.sortValue > right.sortValue
        }
    }

    func profile(_ session: SavedSession) -> HostProfile? {
        hosts.first { $0.id == session.hostID }?.profile(session: session)
    }

    mutating func append(_ profile: HostProfile) {
        let host: SavedHost
        if let existing = hosts.first(where: { $0.address == profile.address && $0.login == profile.login }) {
            host = existing
        } else {
            host = SavedHost(id: UUID(), address: profile.address, login: profile.login)
            hosts.append(host)
        }
        sessions.append(SavedSession(id: profile.id, hostID: host.id, name: profile.session))
    }

    mutating func update(_ host: SavedHost) throws {
        guard let index = hosts.firstIndex(where: { $0.id == host.id }) else { throw ProfileError.missingHost }
        guard !hosts.contains(where: { $0.id != host.id && $0.address == host.address && $0.login == host.login })
        else { throw ProfileError.duplicateHost }
        if hosts[index].address != host.address {
            for i in sessions.indices where sessions[i].hostID == host.id {
                sessions[i].clearObservationForChangedTarget()
            }
        }
        hosts[index] = host
    }
}

// All read/modify/write operations, including migration, share the app's main actor.
@MainActor
struct ProfileStore {
    private static let promotedProfiles = PassthroughSubject<Set<HostProfile>, Never>()
    static var verifiedPhoneKeyProfiles: AnyPublisher<Set<HostProfile>, Never> {
        promotedProfiles.eraseToAnyPublisher()
    }

    static let catalogKey = "sshHostCatalog"
    private static let legacyKey = "sshSessionProfiles"
    let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) { self.defaults = defaults }

    func catalog() throws -> HostCatalog {
        if let object = defaults.object(forKey: Self.catalogKey) {
            guard let data = object as? Data else { throw ProfileError.invalidCatalog }
            return try JSONDecoder().decode(HostCatalog.self, from: data)
        }
        var result = HostCatalog()
        if let object = defaults.object(forKey: Self.legacyKey) {
            guard let data = object as? Data else { throw ProfileError.invalidCatalog }
            for profile in try JSONDecoder().decode([HostProfile].self, from: data) { result.append(profile) }
            try write(result)
            defaults.removeObject(forKey: Self.legacyKey)
        }
        return result
    }

    func load() throws -> [HostProfile] { try catalog().profiles }

    func save(_ profile: HostProfile) throws {
        var catalog = try catalog()
        if let index = catalog.sessions.firstIndex(where: { $0.id == profile.id }),
           var host = catalog.hosts.first(where: { $0.id == catalog.sessions[index].hostID }) {
            if catalog.sessions[index].name != profile.session {
                guard !catalog.sessions.contains(where: { $0.hostID == host.id && $0.id != profile.id && $0.name == profile.session })
                else { throw ProfileError.duplicateSession }
                catalog.sessions[index].clearObservationForChangedTarget()
            }
            host.address = profile.address
            host.login = profile.login
            try catalog.update(host)
            catalog.sessions[index].name = profile.session
        } else {
            guard !catalog.profiles.contains(where: { $0.address == profile.address && $0.login == profile.login && $0.session == profile.session })
            else { throw ProfileError.duplicateSession }
            catalog.append(profile)
        }
        try write(catalog)
    }

    func saveHost(_ host: SavedHost) throws {
        var catalog = try catalog()
        try catalog.update(host)
        try write(catalog)
    }

    func saveSession(on hostID: UUID, id: UUID?, name: String) throws {
        let catalog = try catalog()
        guard let host = catalog.hosts.first(where: { $0.id == hostID }),
              id == nil || catalog.sessions.contains(where: { $0.id == id && $0.hostID == hostID })
        else { throw ProfileError.missingHost }
        let profile = try HostProfile.make(id: id ?? UUID(), hostname: host.address.hostname,
                                           port: String(host.address.port), username: host.address.username,
                                           session: name, login: host.login)
        try save(profile)
    }

    func usePhoneKey(for host: SavedHost) throws -> Bool {
        var catalog = try catalog()
        guard let index = catalog.hosts.firstIndex(where: { $0.id == host.id && $0.address == host.address })
        else { throw ProfileError.missingHost }
        if catalog.hosts[index].login == .phoneKey { return true }
        let hasKeyGroup = catalog.hosts.contains {
            $0.login == .phoneKey && $0.address.username == host.address.username &&
            $0.address.pinAccount == host.address.pinAccount
        }
        guard !hasKeyGroup else { return false }
        let previous = catalog.hosts[index]
        let promoted = Set(catalog.sessions.filter { $0.hostID == previous.id }.map(previous.profile(session:)))
        catalog.hosts[index].login = .phoneKey
        try write(catalog)
        // Every window updates its exact warm connections before SwiftUI reconciles the new catalog.
        Self.promotedProfiles.send(promoted)
        return true
    }

    func mark(_ action: AgentMarkAction, row: HomeAgent) throws {
        var catalog = try catalog()
        guard let session = catalog.sessions.firstIndex(where: { $0.id == row.profile.id }),
              catalog.profile(catalog.sessions[session]) == row.profile,
              var snapshot = catalog.sessions[session].lastStatus, var rows = snapshot.agents,
              let i = rows.firstIndex(where: {
                  $0.bootId == row.agent.bootId && $0.paneId == row.agent.paneId && $0.name == row.agent.name
              }) else { throw ProfileError.missingAgent }
        if case .snooze = action, rows[i].status != row.agent.status || rows[i].stateChangeSeq != row.agent.stateChangeSeq {
            throw ProfileError.missingAgent
        }
        var marks = rows[i].marks ?? AgentMarks()
        marks.apply(action, status: rows[i].status)
        if rows[i].marks == nil && (marks.pinned || marks.snoozeUntil != nil),
           rows.filter({ $0.marks != nil }).count >= SeenAgent.markLimit {
            throw ProfileError.tooManyMarks
        }
        rows[i].marks = marks.pinned || marks.snoozeUntil != nil ? marks : nil
        snapshot.agents = rows
        catalog.sessions[session].lastStatus = snapshot
        try write(catalog)
    }

    func recordSeen(profile: HostProfile, snapshot: SessionStatusSnapshot) throws {
        try record(profile: profile, snapshot: snapshot, opened: true)
    }

    func recordObserved(profile: HostProfile, snapshot: SessionStatusSnapshot) throws {
        try record(profile: profile, snapshot: snapshot, opened: false)
    }

    private func record(profile: HostProfile, snapshot: SessionStatusSnapshot, opened: Bool) throws {
        var catalog = try catalog()
        guard let index = catalog.sessions.firstIndex(where: { $0.id == profile.id }),
              let saved = catalog.profile(catalog.sessions[index]),
              saved.address == profile.address, saved.session == profile.session,
              opened || saved.login == profile.login else { return }
        let advancesOpen = opened && (catalog.sessions[index].lastOpened.map { $0 < snapshot.seenAt } ?? true)
        if advancesOpen { catalog.sessions[index].lastOpened = snapshot.seenAt }
        if let last = catalog.sessions[index].lastStatus, last.seenAt > snapshot.seenAt {
            if advancesOpen { try write(catalog) }
            return
        }
        var snapshot = snapshot
        if let previous = catalog.sessions[index].lastStatus, let rows = snapshot.agents {
            let oldRows = Dictionary((previous.agents ?? []).map { ($0.paneId, $0) }, uniquingKeysWith: { first, _ in first })
            snapshot.agents = rows.map { current in
                var current = current
                guard let old = oldRows[current.paneId], old.bootId == current.bootId, old.name == current.name
                else { return current }
                current.marks = old.marks?.observed(status: current.status, at: snapshot.seenAt)
                if let seq = old.stateChangeSeq, seq > 0, seq == current.stateChangeSeq,
                   PaneActivity(old.status) == .idle, PaneActivity(current.status) == .idle {
                    // A matching daemon sequence confirms the streak across a detached session.
                    // Build 8 did not record whether its first observation witnessed a transition.
                    let oldStart = old.idleOnsetObserved == nil ? previous.seenAt : old.changeObservedAt ?? previous.seenAt
                    let newStart = current.changeObservedAt ?? snapshot.seenAt
                    let oldWitnessed = old.idleOnsetObserved == true
                    let newWitnessed = current.idleOnsetObserved == true
                    if (oldWitnessed && !newWitnessed) || (oldWitnessed == newWitnessed && oldStart <= newStart) {
                        current.changeObservedAt = oldStart
                        current.idleOnsetObserved = oldWitnessed
                    }
                }
                return current
            }
        }
        if let rows = snapshot.agents {
            snapshot.agents = SeenAgent.cache(rows)
        }
        guard advancesOpen || catalog.sessions[index].lastStatus != snapshot else { return }
        catalog.sessions[index].lastStatus = snapshot
        try write(catalog)
    }

    func remove(_ profile: HostProfile) throws {
        var catalog = try catalog()
        catalog.sessions.removeAll { $0.id == profile.id }
        try write(catalog)
    }

    func removeHost(_ id: UUID) throws {
        var catalog = try catalog()
        catalog.hosts.removeAll { $0.id == id }
        catalog.sessions.removeAll { $0.hostID == id }
        try write(catalog)
    }

    private func write(_ catalog: HostCatalog) throws {
        defaults.set(try JSONEncoder().encode(catalog), forKey: Self.catalogKey)
        if defaults === UserDefaults.standard {
            AgentWidgetSnapshot.publish(catalog.agentWidgetSnapshot)
            SpaceportAppShortcuts.updateAppShortcutParameters()
        }
    }
}
