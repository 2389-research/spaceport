// ABOUTME: Presents saved SSH sessions and supports explicit host key management.
// ABOUTME: Each window scene owns its recently opened SSH sessions.
import Darwin
import SwiftUI
import UIKit

@main
struct SpaceportMobileApp: App {
    init() {
        // Rust's executable startup normally makes a closed pipe a write error.
        // This Swift process hosts the same worker and cancellation pipes.
        _ = signal(SIGPIPE, SIG_IGN)
    }
    var body: some Scene { WindowGroup { HostList() } }
}

extension Binding where Value == String? {
    var presented: Binding<Bool> {
        Binding<Bool>(get: { wrappedValue != nil }, set: { visible in
            guard !visible, let previous = wrappedValue else { return }
            // SwiftUI dismisses alerts during presentation updates.
            Task { @MainActor in
                if wrappedValue == previous { wrappedValue = nil }
            }
        })
    }
}
