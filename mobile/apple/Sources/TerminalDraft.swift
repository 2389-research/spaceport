// ABOUTME: Keeps unsent terminal text and its intended pane or popup in memory.
// ABOUTME: Clears a draft only after the client accepts its single paste event.
struct DraftDestination: Equatable {
    let target: InputTarget
    let name: String
    let path: String
    let status: String
    var label: String { [name, path].filter { !$0.isEmpty }.joined(separator: " · ") }

    init(target: InputTarget, workspaces: [MobileWorkspace], tabs: [MobileTab], panes: [MobilePane]) {
        self.target = target
        let pane = panes.first { $0.id == target.paneId }
        let tab = tabs.first { $0.id == pane?.tabId }
        let workspace = workspaces.first { $0.id == tab?.workspaceId }
        path = [workspace?.label, tab?.label].compactMap { $0 }.filter { !$0.isEmpty }.joined(separator: " › ")
        status = pane?.status ?? ""
        var name = target.paneId
        if let pane {
            let paneName = pane.agentName.flatMap { $0.isEmpty ? nil : $0 } ?? (pane.label.isEmpty ? target.paneId : pane.label)
            let duplicate = panes.contains { $0.id != pane.id && ($0.agentName ?? $0.label) == paneName }
            name = duplicate ? "\(paneName) (\(pane.id))" : paneName
        }
        self.name = target.popupId.map { "\(name) › Popup \($0)" } ?? name
    }
}

struct TerminalDraft {
    var text = ""
    var file: DraftFile?
    var image: DraftImage?
    var isEmpty: Bool { text.isEmpty && image == nil && file == nil }
    private(set) var destination: DraftDestination?

    mutating func open(at current: DraftDestination?) {
        if !isEmpty {
            guard let current, var original = destination?.target else { return }
            // Opening Compose again can renew input for this exact terminal.
            original.inputEpoch = current.target.inputEpoch
            guard original == current.target else { return }
        }
        destination = current
    }

    mutating func retarget(to current: DraftDestination) {
        destination = current
    }

    func canSend(to current: InputTarget?) -> Bool {
        guard let target = destination?.target else { return false }
        return !isEmpty && target == current
    }

    mutating func send(to current: InputTarget?, enqueuePaste: (String, InputTarget) -> Bool) -> Bool {
        guard image == nil, file == nil, let current, canSend(to: current), enqueuePaste(text, current) else { return false }
        self = TerminalDraft()
        return true
    }
}
