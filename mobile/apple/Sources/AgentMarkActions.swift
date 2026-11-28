// ABOUTME: Exposes local agent marks through Home row menus and swipe actions.
// ABOUTME: Offers common snooze lengths and a way to bring an agent back early.
import SwiftUI

struct AgentMarkActions: ViewModifier {
    let row: HomeAgent
    let snoozed: Bool
    let change: (AgentMarkAction) -> Void

    private var pinned: Bool { row.agent.marks?.pinned == true }
    private var pin: some View {
        Button(pinned ? "Unpin" : "Pin", systemImage: pinned ? "pin.slash" : "pin") { change(.pin(!pinned)) }
    }
    private var wake: some View {
        Button("Wake now", systemImage: "bell") { change(.wake) }
    }
    private func snooze(hours: Int) { change(.snooze(Date().addingTimeInterval(Double(hours) * 3600))) }

    func body(content: Content) -> some View {
        content
            .contextMenu {
                pin
                if row.agent.status == "done" {
                    let settled = row.agent.marks?.settled == true
                    Button(settled ? "Return to review" : "Mark reviewed", systemImage: "checkmark.circle") {
                        change(.settle(!settled))
                    }
                }
                if snoozed { wake }
                Menu("Snooze", systemImage: "moon.zzz") {
                    Button("For 1 hour") { snooze(hours: 1) }
                    Button("For 3 hours") { snooze(hours: 3) }
                    Button("Until tomorrow at 9 AM") {
                        if let tomorrow = Calendar.current.date(byAdding: .day, value: 1, to: Date()),
                           let morning = Calendar.current.date(bySettingHour: 9, minute: 0, second: 0, of: tomorrow) {
                            change(.snooze(morning))
                        }
                    }
                }
            }
            .swipeActions(edge: .leading, allowsFullSwipe: false) { pin.tint(.blue) }
            .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                if snoozed { wake.tint(.indigo) }
                else { Button("Snooze 1h", systemImage: "moon.zzz") { snooze(hours: 1) }.tint(.indigo) }
            }
    }
}
