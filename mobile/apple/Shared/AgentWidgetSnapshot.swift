// ABOUTME: Shares one session’s observed agent counts between Spaceport and its Home Screen widget.
// ABOUTME: Links each widget entry to the saved session that supplied its counts.
import Foundation
import WidgetKit

struct AgentWidgetSnapshot: Codable, Equatable {
    static let widgetKind = "SpaceportAgentStatus"
    private static let key = "lastAgentObservation"

    let destinationID: UUID
    let counts: AgentCounts
    let observedAt: Date

    private static var sharedDefaults: UserDefaults? {
        guard let group = Bundle.main.object(forInfoDictionaryKey: "SpaceportWidgetGroup") as? String,
              FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: group) != nil else { return nil }
        return UserDefaults(suiteName: group)
    }

    static func load() -> Self? {
        sharedDefaults.flatMap { load(from: $0) }
    }

    static func load(from defaults: UserDefaults) -> Self? {
        guard let data = defaults.data(forKey: key) else { return nil }
        return try? JSONDecoder().decode(Self.self, from: data)
    }

    static func publish(_ snapshot: Self?) {
        guard let defaults = sharedDefaults, store(snapshot, in: defaults) else { return }
        WidgetCenter.shared.reloadTimelines(ofKind: widgetKind)
    }

    static func store(_ snapshot: Self?, in defaults: UserDefaults) -> Bool {
        if let snapshot {
            guard load(from: defaults) != snapshot, let data = try? JSONEncoder().encode(snapshot) else { return false }
            defaults.set(data, forKey: key)
        } else {
            guard defaults.object(forKey: key) != nil else { return false }
            defaults.removeObject(forKey: key)
        }
        return true
    }

    var destinationURL: URL? { Self.destinationURL(for: destinationID) }

    private static func destinationURL(for session: UUID) -> URL? {
        URL(string: "spaceport-widget://next-waiting/\(session.uuidString)")
    }

    static func destinationID(from url: URL) -> UUID? {
        guard let id = UUID(uuidString: url.lastPathComponent), url == destinationURL(for: id) else { return nil }
        return id
    }
}
