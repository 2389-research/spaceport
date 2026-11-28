// ABOUTME: Reads one bounded local file and stages an incoming share in the private app group.
// ABOUTME: The composer takes the file only after the user chooses a destination.
import Foundation

struct DraftFile: Identifiable, Codable {
    let id: UUID
    let name: String
    let data: Data
    static let limit = 16 * 1024 * 1024

    init(url: URL, suggestedName: String? = nil) throws {
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        guard try url.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile == true else {
            throw FileError.unreadable
        }
        let name = suggestedName ?? url.lastPathComponent
        guard !name.isEmpty, ![".", ".."].contains(name), name.utf8.count <= 255,
              !name.contains("/"), !name.unicodeScalars.contains(where: CharacterSet.controlCharacters.contains)
        else { throw FileError.unreadable }
        let handle = try FileHandle(forReadingFrom: url)
        defer { try? handle.close() }
        var bytes = Data()
        while let chunk = try handle.read(upToCount: min(64 * 1024, Self.limit - bytes.count + 1)), !chunk.isEmpty {
            bytes.append(chunk)
            guard bytes.count <= Self.limit else { throw FileError.tooLarge }
        }
        self.id = UUID()
        self.name = name
        self.data = bytes
    }

    enum FileError: LocalizedError {
        case unreadable, tooLarge, pending
        var errorDescription: String? {
            switch self {
            case .unreadable: return "Choose a readable file."
            case .tooLarge: return "Choose a file up to 16 MB."
            case .pending: return "Attach or discard the previous shared file in Spaceport first."
            }
        }
    }
}

enum SharedDraftInbox {
    static var directory: URL? {
        guard let name = Bundle.main.object(forInfoDictionaryKey: "SpaceportWidgetGroup") as? String else { return nil }
        return FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: name)
    }

    static func pending(in directory: URL) -> URL? {
        let inbox = directory.appendingPathComponent("incoming-draft", isDirectory: true)
        return (try? FileManager.default.contentsOfDirectory(at: inbox, includingPropertiesForKeys: nil))?.first
    }

    static func save(_ file: DraftFile, in directory: URL) throws {
        let manager = FileManager.default
        var temporary = directory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try manager.createDirectory(at: temporary, withIntermediateDirectories: false)
        defer { try? manager.removeItem(at: temporary) }
        var flags = URLResourceValues()
        flags.isExcludedFromBackup = true
        try temporary.setResourceValues(flags)
        let encoder = PropertyListEncoder()
        encoder.outputFormat = .binary
        try encoder.encode(file).write(to: temporary.appendingPathComponent(UUID().uuidString), options: .completeFileProtection)
        let inbox = directory.appendingPathComponent("incoming-draft", isDirectory: true)
        guard rename(temporary.path, inbox.path) == 0 else {
            if errno == ENOTEMPTY || errno == EEXIST { throw DraftFile.FileError.pending }
            throw NSError(domain: NSPOSIXErrorDomain, code: Int(errno))
        }
    }

    static func remove(_ entry: URL) throws {
        try FileManager.default.removeItem(at: entry)
        // rmdir never removes a newer, nonempty inbox published by a concurrent share.
        _ = rmdir(entry.deletingLastPathComponent().path)
    }

    static func read(_ url: URL) throws -> DraftFile {
        let size = try url.resourceValues(forKeys: [.fileSizeKey]).fileSize
        guard let size, size <= DraftFile.limit + 4096 else { throw DraftFile.FileError.tooLarge }
        return try PropertyListDecoder().decode(DraftFile.self, from: Data(contentsOf: url))
    }
}
